#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use global_guard::GlobalValue;
use std::mem::size_of;
use comm::LockT;
use config;
use config::Config;
use comm;
use std::ptr;
use shmemx;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct GlobalGuardVec<T: Clone> {
    rank: usize,
    ptr: *mut u8,
    size: usize,
    refer_type: PhantomData<T>
}

// Implement global guard array
impl<T: Clone> GlobalGuardVec<T> {
    pub fn init(config: &mut Config, size: usize) -> GlobalGuardVec<T> {
        let unit = size_of::<T>() + comm::LOCK_SIZE;
        let (ptr, _) = config.alloc(size * unit);

        for i in 0..size {
        	let lock = unsafe{ptr.add(unit * i)} as *mut LockT;
        	comm::clear_lock(lock, config.rank);
        }

        GlobalGuardVec {
            rank: config.rank,
            ptr,
            size,
            refer_type: PhantomData
        }

    }

    pub fn null() -> GlobalGuardVec<T> {
        GlobalGuardVec {
            rank: 0,
            ptr: ptr::null_mut() as *mut u8,
            size: 0,
            refer_type: PhantomData
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn lock(&self, idx: usize) -> GlobalValue<T> {
        if idx >= self.size {
            panic!("GlobalGuardVec out of bound!");
        }
    	unsafe {
    		let offset = idx * (size_of::<T>() + comm::LOCK_SIZE);
	        let lock = unsafe{self.ptr.add(offset)} as *mut LockT;
	        comm::set_lock(lock, self.rank);

	        let rank = self.rank;
	        let ptr = self.ptr.add(offset);
	        GlobalValue::init(rank, ptr)
    	}
        
    }
}

pub struct GuardArray<T: Clone> {
    ptrs: Vec<GlobalGuardVec<T>>,
}

impl<T: Clone> GuardArray<T> {
    pub fn init(config: &mut Config, n:usize) -> GuardArray<T> {
        let local_size = (n + config.rankn - 1) / config.rankn;
        let mut ptrs = vec!(GlobalGuardVec::null(); config.rankn);
        ptrs[config.rank] = GlobalGuardVec::init(config, local_size);

        for rank in 0..config.rankn {
            comm::broadcast(&mut ptrs[rank], rank);
        }

        GuardArray {ptrs}
    }
    pub fn read(&self, idx: usize) -> T {
        // let local_size = (n + shmemx::n_pes() - 1) / config.rankn;
        let local_size = self.ptrs[0].len();
        let rank: usize = idx / local_size;
        // changed to >= by lfz
        if rank >= shmemx::n_pes() {
            panic!("Array::read: index {} out of bound!", idx);
        }
        let local_idx: usize = idx % local_size; // mod % is enough
        let globalval = self.ptrs[rank].lock(local_idx);
        globalval.rget()
    }

    pub fn write(&mut self, c: T, idx: usize) {
        // let local_size = (n + shmemx::n_pes() - 1) / config.rankn;
        let local_size = self.ptrs[0].len();
        let rank: usize = idx / local_size;
        // changed to >= by lfz
        if rank >= shmemx::n_pes() {
            panic!("Array::write: index {} out of bound!", idx);
        }
        let local_idx = idx % local_size; // mod % is enough
        self.ptrs[rank].lock(local_idx).rput(c);

    }

    pub fn lock(&mut self, idx: usize) -> GlobalValue<T> {
        // let local_size = (n + shmemx::n_pes() - 1) / config.rankn;
        let local_size = self.ptrs[0].len();
        let rank: usize = idx / local_size;
        // changed to >= by lfz
        if rank >= shmemx::n_pes() {
            panic!("Array::lock: index {} out of bound! self.ptrs.len(): {}", idx, local_size);
        }
        let local_idx = idx % local_size; // mod % is enough
        self.ptrs[rank].lock(local_idx)
    }
}
