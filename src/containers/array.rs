#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use backend::{comm, shmemx::{self, shmem_broadcast64}};
use base::{config::{self, Config}, global_pointer::{self, GlobalPointer, Bclable}};
use std::marker::PhantomData;

pub struct Array<T: Bclable>{
    pub local_size: usize,
    pub ptrs: Vec<GlobalPointer<T>>,
}
impl <'a, T: Bclable> Array<T>
    where T: Clone + Copy + Default
{
    pub fn init(config: &mut Config, n:usize) -> Array<T> {
        let local_size = (n + config.rankn - 1) / config.rankn;
        let mut ptrs = vec!(GlobalPointer::null(); config.rankn);
        ptrs[config.rank] = GlobalPointer::init(config, local_size);

        for rank in 0..config.rankn {
            comm::broadcast(&mut ptrs[rank], rank);
        }
        Array {local_size, ptrs}
    }

    pub fn read(&self, idx: usize) -> T {
        let rank: usize = idx / self.local_size;
        if rank >= shmemx::n_pes() {
            panic!("Array::read: index {} out of bound!", idx);
        }
        let local_idx: usize = idx % self.local_size; // mod % is enough
        return self.ptrs[rank].idx_rget(local_idx as isize);
    }

    pub fn write(&mut self, c: T, idx: usize) {
        let rank: usize = idx / self.local_size;
        if rank >= shmemx::n_pes() {
            panic!("Array::read: index {} out of bound!", idx);
        }
        let local_idx = idx % self.local_size; // mod % is enough
        self.ptrs[rank].idx_rput(local_idx as isize, c);
    }
    pub fn get_ptr(&self, idx: usize) -> GlobalPointer<T> {
        let rank: usize = idx / self.local_size;
        if rank >= shmemx::n_pes() {
            panic!("Array::read: index {} out of bound!", idx);
        }
        let local_idx: usize = idx % self.local_size; // mod % is enough
        return self.ptrs[rank] + local_idx as isize;
    }

}