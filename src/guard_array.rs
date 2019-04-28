#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use global_guard;
use std::mem::size_of;
use comm::LockT;
use config;
use config::Config;
use comm;
use shmemx;
use std::marker::PhantomData;
use global_pointer::GlobalPointer;
use shmemx::shmem_broadcast64;


struct

struct GlobalGuardVec<T> {
    rank: usize,
    ptr: *const u8,
    size: usize,
    offset: usize
}

// Implement global guard array
impl<T> GlobalGuardVec<T> {
    pub fn init(config: &mut Config, size: usize) -> GlobalGuardVec<T> {
        let (_, offset) = config.alloc(size * size_of::<T>() + comm::LOCK_SIZE * size);

        GlobalGuardVec {
            rank: config.rank,
            ptr: config.alloc(size_of::<T>() + comm::LOCK_SIZE),
            size: 0,
            offset
        }

    }

    pub fn null() -> GlobalGuardVec {
        GlobalGuardVec {
            rank: 0,
            ptr: ptr::null_mut() as *mut u8,
            size: 0
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    pub fn lock(&self, idx: usize) -> GlobalValue<T> {
        let lock = self.ptr as *mut LockT;
        comm::set_lock(lock, self.rank);

        GlobalValue {
            rank: self.rank,
            ptr: self.ptr + idx * size_of::<T>,
            refer_type: PhantomData
        }
    }
}

pub struct SafeArray<T> {
    pub local_size: usize,
    pub ptrs: Vec<GlobalGuardVec>,
}

impl<T> SafeArray<T> {
    pub fn
}
