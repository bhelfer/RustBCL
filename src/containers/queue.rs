#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use backend::{comm, shmemx::{self, shmem_broadcast64, libc::{c_long, c_void, c_int}}};
use base::{config::{self, Config}, global_pointer::{self, GlobalPointer, Bclable}};
use containers::array::Array;

use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// Circular Queue
#[derive(Debug, Clone)]
pub struct Queue<T: Bclable> {
    local_size: usize,
    ptrs: Vec<GlobalPointer<T>>,
    capacity: usize,
    tail_ptr: GlobalPointer<i32>,
    head_ptr: GlobalPointer<i32>,
    slow_tail_ptr: GlobalPointer<i32>, // Used to prevent pop before finishing pushing
}

impl<T: Bclable> Queue<T> {
    pub fn new(config: &mut Config, n: usize) -> Queue<T> {
        if n >= std::i32::MAX as usize {
            panic!("Please set the size of queue less than {}!", std::i32::MAX);
        }
        let mut tail_ptr: GlobalPointer<i32> = GlobalPointer::init(config, 1);
        let mut head_ptr: GlobalPointer<i32> = GlobalPointer::init(config, 1);
        let mut slow_tail_ptr: GlobalPointer<i32> = GlobalPointer::init(config, 1);
        if config.rank == 0 {
            unsafe {
                tail_ptr.local_mut().write(0);
                head_ptr.local_mut().write(0);
                slow_tail_ptr.local_mut().write(0);
            }
        }
        comm::broadcast(&mut tail_ptr, 0);
        comm::broadcast(&mut head_ptr, 0);
        comm::broadcast(&mut slow_tail_ptr, 0);

        let mut ptrs: Vec<GlobalPointer<T>> = Vec::new();
        ptrs.resize(config.rankn, GlobalPointer::null());
        let mut local_size = (n + shmemx::n_pes() - 1) / config.rankn;
        ptrs[config.rank] = GlobalPointer::init(config, local_size);
        for rank in 0..config.rankn {
            comm::broadcast(&mut ptrs[rank], rank);
        }
        Queue { ptrs: ptrs, capacity: n, tail_ptr: tail_ptr, head_ptr: head_ptr, local_size: local_size, slow_tail_ptr: slow_tail_ptr }
    }

    fn get_pointer(&self, global_idx: usize) -> GlobalPointer<T> {
        let rank = global_idx / self.local_size;
        let local_idx = global_idx - rank * self.local_size;
        (self.ptrs[rank] + local_idx as isize)
    }


    pub fn push(&mut self, data: T) -> bool {
        let mut tail = comm::int_atomic_fetch_inc(&mut self.tail_ptr) as usize;
        let head = comm::int_atomic_fetch(&mut self.head_ptr) as usize;
        if tail - head > self.capacity {
            println!("The buffer is full!");
            comm::int_atomic_fetch_add(&mut self.tail_ptr, -1);
            return false;
        }
        tail = tail % self.capacity;
        let rank = tail / self.local_size;
        let local_idx = tail - rank * self.local_size;
        (self.ptrs[rank] + local_idx as isize).rput(data);
        comm::int_atomic_fetch_inc(&mut self.slow_tail_ptr);
        return true;
    }

    pub fn pop(&mut self) -> Result<T, &str> {
        let mut head = comm::int_atomic_fetch_inc(&mut self.head_ptr) as usize;
        let tail = comm::int_atomic_fetch(&mut self.tail_ptr) as usize;
        if tail <= head {
            comm::int_atomic_fetch_add(&mut self.head_ptr, -1);
            return Err("The buffer is empty!");
        } else {
            let slow_tail = comm::int_atomic_fetch(&mut self.slow_tail_ptr) as usize;
            if slow_tail <= head {
                comm::int_atomic_fetch_add(&mut self.head_ptr, -1);
                return Err("The data has not been written yet!");
            }
            head = head % self.capacity;
            let rank = head / self.local_size;
            let local_idx = head - rank * self.local_size;
            return Ok((self.ptrs[rank] + local_idx as isize).rget());
        }
    }

    pub fn peek(&mut self) -> Result<T, &str> {
        let mut head = comm::int_atomic_fetch(&mut self.head_ptr) as usize;
        let tail = comm::int_atomic_fetch(&mut self.tail_ptr) as usize;
        if tail <= head {
            return Err("The buffer is empty!");
        } else {
            let slow_tail = comm::int_atomic_fetch(&mut self.slow_tail_ptr) as usize;
            if slow_tail <= head {
                return Err("The data has not been written!");
            }
            head = head % self.capacity;
            let rank = head / self.local_size;
            let local_idx = head - rank * self.local_size;
            return Ok((self.ptrs[rank] + local_idx as isize).rget());
        }
    }

    pub fn len(&mut self) -> usize {
        let head = comm::int_atomic_fetch(&mut self.head_ptr) as usize;
        let tail = comm::int_atomic_fetch(&mut self.tail_ptr) as usize;
        return tail - head;
    }

    pub fn is_empty(&mut self) -> bool {
        return self.len() == 0;
    }

    pub fn clear(&mut self) {
        self.head_ptr.rput(0);
        self.tail_ptr.rput(0);
        self.slow_tail_ptr.rput(0);
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}