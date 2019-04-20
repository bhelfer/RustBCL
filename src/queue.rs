#![allow(dead_code)]
#![allow(unused)]

use global_pointer;
use comm;
use config;
use config::Config;
use shmemx;
use global_pointer::GlobalPointer;
use array::Array;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// Circular Queue
#[derive(Debug, Clone)]
pub struct Queue<T> {
    local_size: usize,
    ptrs: Vec<GlobalPointer<T>>,
    capacity: usize,
    tail_ptr: GlobalPointer<i32>,
    head_ptr: GlobalPointer<i32>,
}

impl<'a, T: Clone + Copy + Default> Queue<T> {
    pub fn new(config: &mut Config, n: usize) -> Queue<T> {
        let mut tail_ptr = config.alloc::<i32>(1);
        let mut head_ptr = config.alloc::<i32>(1);
        if config.rank == 0 {
            unsafe {
                tail_ptr.local().write(0);
                head_ptr.local().write(0);
            }
        }
        comm::broadcast(&mut tail_ptr, 0);
        comm::broadcast(&mut head_ptr, 0);

        let mut ptrs: Vec<GlobalPointer<T>> = Vec::new();
        ptrs.resize(config.rankn, GlobalPointer::null());
        let mut local_size = (n + shmemx::n_pes() - 1) / config.rankn;
        ptrs[config.rank] = config.alloc::<T>(local_size);
        for rank in 0..config.rankn {
            comm::broadcast(&mut ptrs[rank], rank);
        }
        Queue { ptrs: ptrs, capacity: n, tail_ptr: tail_ptr, head_ptr: head_ptr, local_size: local_size }

    }

    fn get_pointer(&self, global_idx: usize) -> GlobalPointer<T> {
        let rank = global_idx / self.local_size;
        let local_idx = global_idx - rank * self.local_size;
        (self.ptrs[rank] + local_idx)
    }


    pub fn add(&mut self, data: T) -> bool {
        let mut tail = comm::int_finc(&mut self.tail_ptr) as usize;
        let head = self.head_ptr.rget() as usize;
        if tail - head > self.capacity {
            panic!("The buffer is full!");
            return false;
        }
        tail = tail % self.capacity;
        let rank = tail / self.local_size;
        let local_idx = tail - rank * self.local_size;
//        println!("Add elemetn: tail: {}, rank :{}, local_idx: {}", tail, rank, local_idx);
        (self.ptrs[rank] + local_idx).rput(data);
        return true;
    }

    pub fn remove(&mut self) -> Result<T, &str> {
        let mut head = comm::int_finc(&mut self.head_ptr) as usize;
        let tail = self.tail_ptr.rget() as usize;

        if tail <= head { // TODO test
            Err("The buffer is empty!")
        } else {
            head = head % self.capacity;
            let rank = head / self.local_size;
            let local_idx = head - rank * self.local_size;
//            println!("Remove elemetn: head: {}, rank :{}, local_idx: {}", head, rank, local_idx);
            Ok((self.ptrs[rank] + local_idx).rget())
        }
    }

    pub fn peek(&self) -> Result<T, &str> {
        let mut head = self.head_ptr.rget() as usize;
        let tail = self.tail_ptr.rget() as usize;

        if tail <= head { // TODO test
            Err("The buffer is empty!")
        } else {
            head = head % self.capacity;
            let rank = head / self.local_size;
            let local_idx = head - rank * self.local_size;
            Ok((self.ptrs[rank] + local_idx).rget())
        }
    }

    pub fn len(&self) -> usize {
        let head = self.head_ptr.rget();
        let tail = self.tail_ptr.rget();
        return (tail - head) as usize
    }

    pub fn capacity(&self) ->usize {
        self.capacity
    }
}
