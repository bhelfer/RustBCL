#![allow(dead_code)]
#![allow(unused)]
use global_pointer;
use comm;
use config;
use config::Config;
use shmemx;
use std::marker::PhantomData;
use global_pointer::GlobalPointer;


pub struct Array<T>{
    pub local_size: usize,
    pub ptrs: Vec<GlobalPointer<T>>,
    pub refer_type: PhantomData<T>,
}
impl <'a, T> Array<T> {
    pub fn init(n:usize) -> Array<T>{
        let local_size = (n + shmemx::n_pes() - 1) / shmemx::n_pes();
        let ptrs = Vec(<GlobalPointer<T>>);
        let refer_type = PhantomData;
        Array{ local_size, ptrs, refer_type}
    }
    pub fn array(&mut self, n: usize) {
        self.local_size = (n + shmemx::n_pes() - 1) / shmemx::n_pes();
        //let mut c: Config = config::Config{};
        let mut c: Config = Config::init(n);

        //self.ptrs[shmemx::my_pe()] = c::alloc<T>(self.local_size);
        self.ptrs[shmemx::my_pe()] = c.alloc::<T>(self.local_size);
        //self.local_size = (n + BCL::nprocs() - 1) / BCL::nprocs();
        //self.ptrs[BCL::rank()] = BCL::alloc<char>(self.local_size);
        
        for rank in 0..shmemx::n_pes() {
            let mut ptr1: GlobalPointer<T> = self.ptrs[rank];
            comm::broadcast(&mut ptr1, rank);
            //self.ptrs[rank] = BCL::broadcast(self.ptrs[rank], rank);
        }
    }
    pub fn read(&self, idx: usize) -> T {
        let rank: usize = idx / self.local_size;
        let local_idx: usize = idx - rank*self.local_size;
        return self.ptrs[rank].rget();
        //return self.ptrs[rank]
        //return self.ptrs[rank][local_idx]
        
    }
    pub fn write(&mut self, c: T, idx: usize) {
        let rank: usize = idx / self.local_size;
        let local_idx = idx - rank * self.local_size;
        self.ptrs[rank].rput(c);
    }
}

