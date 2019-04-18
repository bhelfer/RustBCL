#![allow(dead_code)]
#![allow(unused)]
use global_pointer;
use comm;
use config;
use config::Config;
use shmemx;
use std::marker::PhantomData;
use global_pointer::GlobalPointer;
use array::Array;

#[derive(Debug, Copy, Clone)]
pub struct Queue<T> {
    pub size: usize,
    pub refer_type: PhantomData<T>,
    array: Array<T>,
    layer_size: usize,
}

impl<'a, T> Queue<T> {
    pub fn init() -> Queue<T> {
        let queue = Queue{size: 0, refer_type: PhantomData};
    }
}
