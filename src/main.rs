#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

extern crate rand;

mod shmemx;
mod global_pointer;
mod config;
mod comm;
mod global_guard;
use config::Config;
use global_guard::{GlobalGuard, GlobalValue};
use self::rand::{Rng, SeedableRng, StdRng};
use std::collections::HashMap;

fn main() {

    let mut config = Config::init(1);
    let rankn = config.rankn;

    if config.rankn < 2 {
        return;
    }

    test_global_guard(&mut config);
}

fn test_global_guard(config: &mut Config) {
	// ----------- Global Guard's part -------------
    if config.rank == 0 { println!("------------Global Guard's test------------\n"); }

    let mut guard1 = GlobalGuard::null();
    if config.rank == 0 {
        guard1 = GlobalGuard::init(config);
    }
    comm::broadcast(&mut guard1, 0);
    // println!("rank:{}, guard1:{:?}", config.rank, guard1);
    if config.rank == 0 {
    	let value = guard1.lock();
    	value.rput(0);
    }

    // text mutex
    for i in 0..1000 {
    	let value = guard1.lock();
    	let t = value.rget();
    	value.rput(t + 1);
    }
    comm::barrier();

    if config.rank == 0 {
    	let value = guard1.lock();
    	let t = value.rget();
    	assert_eq!(t, 2000);
    	println!("Global Guard's test: pass!");
    }
}