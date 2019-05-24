#![allow(dead_code)]
#![allow(unused)]

use backend::comm::{self, LockT};
use backend::shmemx::{self, libc::{c_int, size_t, c_long, c_void}};
use base::config::Config;

use std::marker::PhantomData;
use std::ops;
use std::mem::size_of;
use std::ptr;

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

    pub fn test_lock(&self) -> Result<GlobalValue<T>, &str> {
        let lock = self.ptr as *mut LockT;
        let success = comm::test_lock(lock, self.rank);
        if success {
        	Ok(GlobalValue {
            rank: self.rank,
            ptr: self.ptr,
            refer_type: PhantomData
        	})
        } else {
        	Err("Cannot get the lock")
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
	pub unsafe fn init(rank: usize, ptr: *mut u8) -> GlobalValue<T> {
		GlobalValue {
			rank,
			ptr,
			refer_type: PhantomData
		}
	}

	pub fn rget(&self) -> T {
        unsafe {
            let mut value: T = std::mem::uninitialized::<T>();
            let len = size_of::<T>();
            let source = self.ptr.add(comm::LOCK_SIZE);
            let target = &mut value as *mut T as *mut u8;
            self.getmem(target, source, len, self.rank);
            value
        }
	}

	pub fn rput(&self, value: T) {
        unsafe{
            let target = self.ptr.add(comm::LOCK_SIZE);
            let source = &value as *const T as *mut u8;
            let len = size_of::<T>();
            self.putmem(target, source, len, self.rank);
        }
	}

    unsafe fn putmem(&self, target: *mut u8, source: *const u8, len: usize, pe: usize) {
    	if shmemx::my_pe() == self.rank {
    		libc::memcpy(target as *mut c_void, source as *const T as *const c_void, len);// * size_of::<T>());
    	} else {
	        shmemx::shmem_putmem(target, source, len as size_t, self.rank as c_int);
    	}
    }

    unsafe fn getmem(&self, target: *mut u8, source: *const u8, len: usize, pe: usize) {
    	if shmemx::my_pe() == self.rank {
    		libc::memcpy(target as *mut c_void, source as *const T as *const c_void, len);//  * size_of::<T>());
    	} else {
     	   shmemx::shmem_getmem(target, source, len as size_t, self.rank as c_int);
    	}
    }
}

impl<T> Drop for GlobalValue<T> {
	fn drop(&mut self) {
        let lock = self.ptr as *mut LockT;
        comm::clear_lock(lock, self.rank);
    }
}