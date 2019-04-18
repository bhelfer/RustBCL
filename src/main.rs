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

    let data = Box::new(String::from("Hello world!"));
    let ptr: serial_ptr<char> = serializer::serialize(data);
    ptr.print();

    config.finalize();
}