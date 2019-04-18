#![allow(dead_code)]
#![allow(unused)]
#![allow(non_camel_case_types)]

#![feature(specialization)]

use std::marker::PhantomData;
use global_pointer::GlobalPointer;
use std::rc::Rc;
use std::iter::FromIterator;

pub trait serializer<R> {
    fn serialize(data: Self) -> R;
    fn deserialize(data: R) -> Self;
}

impl<T: Copy> serializer<T> for T {
    fn serialize(data: T) -> T {
        data
    }
    fn deserialize(data: T) -> T {
        data
    }
}

impl serializer<serial_ptr<char>> for String {
    fn serialize(data: String) -> serial_ptr<char> {
        let mut ptr: serial_ptr<char> = serial_ptr::new(data.len());
        for (i, c) in data.chars().enumerate() {
            ptr.ptr[i] = c;
        }
        ptr
    }

    fn deserialize(ptr: serial_ptr<char>) -> String {
        String::from_iter(ptr.ptr.iter())
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