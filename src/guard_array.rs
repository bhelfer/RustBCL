#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use global_guard::GlobalValue;
use std::mem::size_of;
use comm::LockT;
use config;
use config::Config;
use comm;
use shmemx;
use std::marker::PhantomData;
use global_pointer::GlobalPointer;
use shmemx::shmem_broadcast64;


/*
struct GlobalGuardVec<T> {
    ptrs: Vec<GlobalGuard<T>>,
    local_size: usize
}

impl<T> GlobalGuardVec<T> {
    pub fn init(config: &mut Config, size: usize) -> GlobalGuardVec<T> {

    }

}
*/

struct GlobalGuardVec<T> {
    rank: usize,
    ptr: *const u8,
    size: usize,
    // offset: usize
}

// Implement global guard array
impl<T> GlobalGuardVec<T> {
    pub fn init(config: &mut Config, size: usize) -> GlobalGuardVec<T> {
        let (ptr, _) = config.alloc(size * (size_of::<T>() + comm::LOCK_SIZE));

        GlobalGuardVec {
            rank: config.rank,
            ptr,
            //ptr: config.alloc(size_of::<T>() + comm::LOCK_SIZE),
            size: 0,
            // offset: size_of::<T> + comm::LOCK_SIZE
        }

    }

    pub fn null() -> GlobalGuardVec<T> {
        GlobalGuardVec {
            rank: 0,
            ptr: ptr::null_mut() as *mut u8,
            size: 0,
            // offset: 0
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn lock(&self, idx: usize) -> GlobalValue<T> {
        let offset = idx * (size_of::<T> + comm::LOCK_SIZE);
        let lock = unsafe{self.ptr.add(offset)} as *mut LockT;
        comm::set_lock(lock, self.rank);

        GlobalValue {
            rank: self.rank,
            ptr: unsafe{self.ptr.add(offset)},
            // ptr: self.ptr + idx * size_of::<T>,
            refer_type: PhantomData
        }
    }
}

pub struct SafeArray<T> {
    pub ptrs: Vec<GlobalGuardVec<T>>,
}

impl<'a, T: Clone + Copy + Default> SafeArray<T> {
    pub fn init(config: &mut Config, arr_size:usize) -> SafeArray<T> {
        let local_size = (arr_size + shmemx::n_pes() - 1) / config.rankn;
        let mut ptrs = vec!(GlobalGuardVec::null(); config.rankn);
        // ptrs[config.rank] = config.alloc::<T>(local_size);
        ptrs[config.rank] = ptrs[config.rank].init(config, local_size);

        for rank in 0..config.rankn {
            comm::broadcast(&mut ptrs[rank], rank);
        }
        SafeArray {ptrs}
    }

    pub fn read(&self, idx: usize) -> T {
        let local_size = (arr_size + shmemx::n_pes() - 1) / config.rankn;
        let rank: usize = idx / self.local_size;
        // changed to >= by lfz
        if rank >= shmemx::n_pes() {
            panic!("Array::read: index {} out of bound!", idx);
        }
        let local_idx: usize = idx % self.local_size; // mod % is enough
        return self.ptrs[rank].idx_rget(local_idx as isize);
    }

    pub fn write(&mut self, c: T, idx: usize) {
        let local_size = (arr_size + shmemx::n_pes() - 1) / config.rankn;
        let rank: usize = idx / self.local_size;
        // changed to >= by lfz
        if rank >= shmemx::n_pes() {
            panic!("Array::read: index {} out of bound!", idx);
        }
        let local_idx = idx % self.local_size; // mod % is enough
        self.ptrs[rank].idx_rput(local_idx as isize, c);

    }
}
