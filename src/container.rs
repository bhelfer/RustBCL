#![allow(dead_code)]
#![allow(unused)]
#![allow(non_camel_case_types)]

use std::marker::PhantomData;
use global_pointer::GlobalPointer;
use std::rc::Rc;
use std::iter::FromIterator;

#[derive(Debug, Copy, Clone)]
pub struct serializer<T> {
    pub refer_type: PhantomData<T>
}

impl serializer<String> {
    pub fn serialize(data: Box<String>) -> serial_ptr<char> {
        let data = Box::leak(data);
        let mut ptr: serial_ptr<char> = serial_ptr::new(data.len());
        for (i, c) in data.chars().enumerate() {
            ptr.ptr[i] = c;
        }
        ptr
    }

    pub fn deserialize(ptr: serial_ptr<char>) -> Box<String> {
        Box::new(String::from_iter(ptr.ptr.iter()))
    }
}

#[derive(Debug, Clone)]
pub struct serial_ptr<T> {
    pub ptr: Vec<T>,
    n: usize,
}

impl<T> serial_ptr<T>
    where T: std::clone::Clone + std::default::Default
{
    pub fn new(n: usize) -> serial_ptr<T> {
        let mut ptr: Vec<T> = Vec::new();
        ptr.resize(n, Default::default());
        serial_ptr {
            ptr,
            n,
        }
    }
}

impl<T> serial_ptr<T>
    where T: std::fmt::Debug
{
    pub fn print(&self) {
        for i in 0..self.n {
            print!("{:?}", self.ptr[i]);
        }
        println!();
    }
}