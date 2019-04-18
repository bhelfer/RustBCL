#![allow(dead_code)]
#![allow(unused)]
use config::Config;
use global_pointer::GlobalPointer;
use container::{serial_ptr, serializer};

mod shmemx;
mod global_pointer;
mod config;
mod comm;
mod container;
//use std::io::{self, Write};

fn main() {

    let mut config = Config::init(1);
    Config::barrier();

    let src1: String = String::from("Hello world!");
    let ptr1 = serializer::serialize(src1);
    ptr1.print();

    let dst1: String = serializer::deserialize(ptr1);
    println!("{}", dst1);

    let src2: usize = 233;
    let ptr2 = serializer::serialize(src2);
    let dst2: usize = serializer::deserialize(ptr2);
    println!("{}", dst2);

    config.finalize();
}