#![allow(dead_code)]
#![allow(unused)]
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
//        let ptrs = Vec(<GlobalPointer<T>>);
        let mut ptrs = vec!(GlobalPointer::null(); config.rankn);
        //self.ptrs[shmemx::my_pe()] = c::alloc<T>(self.local_size);
        ptrs[config.rank] = config.alloc::<T>(local_size);
        //self.local_size = (n + BCL::nprocs() - 1) / BCL::nprocs();
        //self.ptrs[BCL::rank()] = BCL::alloc<char>(self.local_size);

        for rank in 0..config.rankn {
//            let mut ptr1: GlobalPointer<T> = self.ptrs[rank];
//            comm::broadcast(&mut ptr1, rank);
            comm::broadcast(&mut ptrs[rank], rank);
        }
//        let refer_type = PhantomData;
//        Array{ local_size, ptrs, refer_type}
        Array{ local_size, ptrs}
    }
//    pub fn array(&mut self, n: usize) {
//        self.local_size = (n + shmemx::n_pes() - 1) / shmemx::n_pes();
//        //let mut c: Config = config::Config{};
//        let mut c: Config = Config::init(n);
//
//        //self.ptrs[shmemx::my_pe()] = c::alloc<T>(self.local_size);
//        self.ptrs[shmemx::my_pe()] = c.alloc::<T>(self.local_size);
//        //self.local_size = (n + BCL::nprocs() - 1) / BCL::nprocs();
//        //self.ptrs[BCL::rank()] = BCL::alloc<char>(self.local_size);
//
//        for rank in 0..shmemx::n_pes() {
//            let mut ptr1: GlobalPointer<T> = self.ptrs[rank];
//            comm::broadcast(&mut ptr1, rank);
//            //self.ptrs[rank] = BCL::broadcast(self.ptrs[rank], rank);
//        }
//    }
    pub fn read(&self, idx: usize) -> T {
        let rank: usize = idx / self.local_size;
        if rank > shmemx::n_pes() {
            panic!("Array::read: index {} out of bound!", idx);
        }
        let local_idx: usize = idx % self.local_size; // mod % is enough
        return self.ptrs[rank].rget();
        //return self.ptrs[rank]
        //return self.ptrs[rank][local_idx]
        
    }
    pub fn write(&mut self, c: T, idx: usize) {
        let rank: usize = idx / self.local_size;
        if rank > shmemx::n_pes() {
            panic!("Array::read: index {} out of bound!", idx);
        }
        let local_idx = idx % self.local_size; // mod % is enough
        self.ptrs[rank].rput(c);
    }
}

