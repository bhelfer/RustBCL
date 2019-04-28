#![allow(dead_code)]
#![allow(unused)]
use comm::LockT;
use shmemx;
use std::marker::PhantomData;
use std::ops;
use std::mem::size_of;
use Config;
use std::ptr;
use shmemx::libc::{c_int, size_t, c_long};
use comm;

#[derive(Debug, Copy, Clone)]
pub struct GlobalGuard<T> {
	rank: usize,
	ptr: *mut u8,  // count by size_of(T)
	refer_type: PhantomData<T>
}

// implement GlobalGuard
impl<T> GlobalGuard<T> {
    pub fn init(config: &mut Config) -> GlobalGuard<T> {
        let (ptr, _) = config.alloc(size_of::<T>() + comm::LOCK_SIZE);

        let lock = ptr as *mut LockT;
        comm::clear_lock(lock, config.rank);

        GlobalGuard {
            rank: config.rank,
            ptr: ptr as *mut u8,
            refer_type: PhantomData
        }
    }

    pub fn null() -> GlobalGuard<T> {
        GlobalGuard {
            rank: 0,
            ptr: ptr::null_mut() as *mut u8,
            refer_type: PhantomData
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    pub fn lock(&self) -> GlobalValue<T> {
        let lock = self.ptr as *mut LockT;
        comm::set_lock(lock, self.rank);

        GlobalValue {
            rank: self.rank,
            ptr: self.ptr,
            refer_type: PhantomData
        }
    }
}

#[derive(Debug)]
pub struct GlobalValue<T> {
	rank: usize,
	ptr: *mut u8,  // count by size_of(T)
	refer_type: PhantomData<T>
}

impl<T> GlobalValue<T> {
	pub fn rget(&self) -> T {
        unsafe {
            let mut value: T = std::mem::uninitialized::<T>();
            let len = size_of::<T>();
            let source = self.ptr.add(comm::LOCK_SIZE);
            let target = &mut value as *mut T as *mut u8;
            shmemx::shmem_getmem(target, source, len as size_t, self.rank as i32);
            value
        }
	}

	pub fn rput(&self, value: T) {
        unsafe{
            let target = self.ptr.add(comm::LOCK_SIZE);
            let source = &value as *const T as *const u8;
            let len = size_of::<T>() as size_t;
            shmemx::shmem_putmem(target, source, len, self.rank as i32);
        }
	}
}

impl<T> Drop for GlobalValue<T> {
	fn drop(&mut self) {
        let lock = self.ptr as *mut LockT;
        comm::clear_lock(lock, self.rank);
    }
}