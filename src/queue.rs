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
pub struct Queue<T> {
    pub len: usize,
    array: Array<T>,
    capacity: usize,
    tail_ptr: GlobalPointer<i32>,
    head_ptr: GlobalPointer<i32>,
//    add_mutex: AtomicBool,
//    remove_mutex: AtomicBool,
}

impl<'a, T: Clone> Queue<T> {
    pub fn new(mut config: &mut Config) -> Queue<T> {
        let rankn = config.rankn;
        let size: usize = 100;
        let mut array = Array::<T>::init(&mut config, size);
        let mut tail_ptr = config.alloc::<i32>(1);
        let mut head_ptr = config.alloc::<i32>(1);
        unsafe {
            tail_ptr.local().write(0);
            head_ptr.local().write(0);
        }
        for rank in 0..config.rank {
            comm::broadcast(&mut tail_ptr, rank);
            comm::broadcast(&mut head_ptr, rank);
        }
        config.barrier();
        Queue{len: 0, array: array, capacity: rankn, tail_ptr: tail_ptr, head_ptr: head_ptr}
    }

    // TODO flexible size
    pub fn add(&mut self, data: T) -> bool {
//        let mut tail_ptr = self.tail_ptr;
        let tail = comm::int_finc(&mut self.tail_ptr);
        let head = self.head_ptr.rget();
        if tail - head > self.capacity as i32 {
            panic!("The buffer is full!");
            return false;
        }
        self.array.write(data, ((tail - 1) as usize) % self.capacity);
        self.len += 1;
        return true;
    }

    pub fn peek(&self) -> Result<T, &str> {
        if self.len > 0 {
            Ok(self.array.read(self.head_ptr.rget() as usize))
        } else {
            Err("The buffer is empty!")
        }
    }

    pub fn remove(&mut self) -> Result<T, &str> {
//        let lock = Arc::new(AtomicBool::new(false));
//        while lock.compare_and_swap(false, true, Ordering::Acquire) { }
//        while self.remove_mutex.load(Ordering::Acquire) {}
//        *self.remove_mutex.get_mut() = true;
        if self.len > 0 {
            let head = comm::int_finc(&mut self.head_ptr);
            self.len -= 1;
//            lock.store(false, Ordering::Release);
//            *self.remove_mutex.get_mut() = false;
            Ok(self.array.read((self.head_ptr.rget() - 1) as usize))
        } else {
//            lock.store(false, Ordering::Release);
//            *self.remove_mutex.get_mut() = false;
            Err("The buffer is empty!")
        }
    }
}
