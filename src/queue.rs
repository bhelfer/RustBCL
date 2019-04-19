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

pub struct Queue<T> {
    pub len: usize,
    array: Array<T>,
    capacity: usize,
    enqueue_idx: usize,
    dequeue_idx: usize,
//    add_mutex: AtomicBool,
//    remove_mutex: AtomicBool,
}

impl<'a, T: Clone> Queue<T> {
    pub fn init(mut config: &mut Config) -> Queue<T> {
        let rankn = config.rankn;
        let mut array = Array::<T>::init(&mut config, rankn);
        Queue{len: 0, array: array, capacity: rankn, enqueue_idx: 0, dequeue_idx: 0}
    }
    pub fn add(&mut self, mut config: &mut Config, data: T) {
//        let lock = Arc::new(AtomicBool::new(false));
//        while lock.compare_and_swap(false, true, Ordering::Acquire) { }

//        while self.add_mutex.load(Ordering::Acquire) {}
//        *self.add_mutex.get_mut() = true;
        if self.capacity <= self.enqueue_idx + 1 {
            let rankn = config.rankn;
            let mut array = Array::<T>::init(&mut config, self.capacity * 2);
            let old_array_size = self.array.local_size * rankn;
            for i in 0..old_array_size {
                array.write(self.array.read(i), i);
            }
            self.array = array;
            self.capacity = self.capacity * 2;
        }
        self.array.write(data, self.enqueue_idx);
        self.enqueue_idx += 1;
        self.len += 1;
//        *self.add_mutex.get_mut() = false;
//        lock.store(false, Ordering::Release);
    }

    pub fn peek(&self) -> Result<T, &str> {
        if self.len > 0 {
            Ok(self.array.read(self.dequeue_idx))
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
            self.dequeue_idx += 1;
            self.len -= 1;
//            lock.store(false, Ordering::Release);
//            *self.remove_mutex.get_mut() = false;
            Ok(self.array.read(self.dequeue_idx - 1))
        } else {
//            lock.store(false, Ordering::Release);
//            *self.remove_mutex.get_mut() = false;
            Err("The buffer is empty!")
        }
    }
}
