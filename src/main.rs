#![allow(dead_code)]

use config::Config;
use global_pointer::GlobalPointer;

mod shmemx;
mod global_pointer;
mod config;
mod comm;

//use std::io::{self, Write};

fn main() {
    let mut config = Config::init(1);
    Config::barrier();

    if config.rankn < 2 {
        config.finalize();
        return;
    }

    let mut ptr1 = GlobalPointer::null();
    if config.rank == 0 {
	    let rankn = config.rankn;
        ptr1 = config.alloc::<i32>(rankn);
    }

    println!("my_rank: {}, ptr before broadcast: {:?}", config.rank, ptr1);
    comm::broadcast(&mut ptr1, 0);
    Config::barrier();
    println!("my_rank: {}, ptr after broadcast: {:?}", config.rank, ptr1);

    // write value
    (ptr1 + config.rank).rput(config.rank as i32);

    let mut value;

    if config.rank == 0 {
        let p1 = ptr1.local();
        let p_slice = unsafe{ std::slice::from_raw_parts(p1, config.rankn) };
        println!("Rank 0 Sees: ");
        for i in 0..config.rankn {
            value = p_slice[i];
            println!("{}: {}", i, value);
        }
    }

    Config::barrier();
    // barrier not work TaT, or just println! is slow?
    println!("barrier1, rank{}!", config.rank);
    Config::barrier();

    if config.rank == 1 {
        println!("Rank 1 Sees: ");
        for i in 0..config.rankn {
            value = (ptr1 + i).rget();
            println!("{}: {}", i, value);
        }
    }

    Config::barrier();

    if config.rank == 0 {
        config.free(ptr1);
    }

    config.finalize();
}
