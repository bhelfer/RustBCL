#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use global_pointer;
use comm;
use config;
use config::Config;
use shmemx;
use std::marker::PhantomData;
use global_pointer::GlobalPointer;
use shmemx::shmem_broadcast64;

pub struct Array<T>{
    pub local_size: usize,
    pub ptrs: Vec<GlobalPointer<T>>,
    // pub refer_type: PhantomData<T>, // JY: since you already use the type T in field ptrs, you do not need this PhantomData.
}
impl <'a, T: Clone> Array<T> {
    /*
    JY:
    My intention with Config is to let it hold all the global variables.
    So every program should only have one unique Config.
    My suggestion is not to init another Config.
    You can either pass the &mut Config to this function,
    or implement the initialization of array as a method of Config, just like the init of GlobalPointer.
    I have implemented the first way in your init function.
    */
    pub fn init(config: &mut Config, n:usize) -> Array<T> {
        let local_size = (n + shmemx::n_pes() - 1) / config.rankn;
        let mut ptrs = vec!(GlobalPointer::null(); config.rankn);
        ptrs[config.rank] = config.alloc::<T>(local_size);

        for rank in 0..config.rankn {
            comm::broadcast(&mut ptrs[rank], rank);
        }
        Array {local_size, ptrs}
    }

    pub fn read(&self, idx: usize) -> T {
        let rank: usize = idx / self.local_size;
        // changed to >= by lfz
        if rank >= shmemx::n_pes() {
            panic!("Array::read: index {} out of bound!", idx);
        }
        let local_idx: usize = idx % self.local_size; // mod % is enough
        return self.ptrs[rank].idx_rget(local_idx as isize);
    }

    pub fn write(&mut self, c: T, idx: usize) {
        let rank: usize = idx / self.local_size;
        // changed to >= by lfz
        if rank >= shmemx::n_pes() {
            panic!("Array::read: index {} out of bound!", idx);
        }
        let local_idx = idx % self.local_size; // mod % is enough
        self.ptrs[rank].idx_rput(local_idx as isize, c);
    }
    pub fn global_size(&mut self, config: &mut Config) -> usize {
        return self.local_size * config.rankn;
    }
    pub fn slice(&mut self, mut begin_idx: usize, mut end_idx: usize) -> Vec<T>{
        let begin_rank: usize = begin_idx/self.local_size;
        let end_rank: usize = end_idx/self.local_size;
        let mut local_slice = Vec::new();
        if begin_rank >= shmemx::n_pes(){
            panic!("Array::read: index {} out of bound!", begin_rank);
        } 
        if end_rank >= shmemx::n_pes(){
            panic!("Array::read: index {} out of bound!", end_rank);
        }
        if begin_rank != end_rank {
            for rank in begin_rank..end_rank {
                for i in begin_idx..self.local_size{
                    local_slice.push(self.ptrs[rank].idx_rget(i as isize));
                }
                begin_idx = 0
            }
            for i in 0..end_idx{
                local_slice.push(self.ptrs[end_rank].idx_rget(i as isize))
            }
        }
        else{
            for i in begin_idx..end_idx{
                //local_slice.push(self.read(i));
                local_slice.push(self.ptrs[begin_rank].idx_rget(i as isize));
            }
        }
        return local_slice

    }

}