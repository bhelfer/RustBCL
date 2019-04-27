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

pub struct Slice<T>{
    pub size: usize,
    pub ptr: GlobalPointer<T>,
    // pub refer_type: PhantomData<T>, // JY: since you already use the type T in field ptrs, you do not need this PhantomData.
}