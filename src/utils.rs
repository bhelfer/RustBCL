#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use std::ops::AddAssign;

pub struct Buf {
    buffer: String
}

impl Buf {
    pub fn new() -> Self {
        Self { buffer: String::new() }
    }
    pub unsafe fn add_buf(&mut self, string: &String) {
        self.buffer.add_assign(string);
    }
    pub unsafe fn clear_buf(&mut self) {
        println!("{}", self.buffer);
        self.buffer.clear();
    }
}