#![allow(dead_code)]
#![allow(unused)]

use global_pointer::GlobalPointer;
use std::rc::Rc;

pub struct serialize<T> {

}

impl serialize<str> {

}

pub struct serial_ptr<T> {
    pub ptr: Rc<T>,
    n: usize,
}

impl<T> serial_ptr<T> {

    pub fn new(n: usize) -> serial_ptr<T> {
        let ptr = Rc::new(T);
        serial_ptr {
            ptr,
            n,
        }
    }

    pub fn print() {
        for i in 0..n {
            print!(ptr[i]);
        }
        println!();
    }
}