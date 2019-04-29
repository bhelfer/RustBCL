#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

extern crate rand;

pub mod shmemx;
pub mod global_pointer;
pub mod config;
pub mod comm;
pub mod array;
pub mod hash_table;
pub mod queue;
pub mod global_guard;

use config::Config;
use global_pointer::GlobalPointer;
use array::Array;
use hash_table::HashTable;
use queue::Queue;
use global_guard::GlobalGuard;

use self::rand::{Rng, SeedableRng, StdRng};
use std::collections::HashMap;

fn main() {

    let mut config = Config::init(1);
    let rankn = config.rankn;

    if config.rankn < 2 {
        return;
    }

//    test_ptr(&mut config);

   // test_global_pointer(&mut config);

//    test_shmem_atomic(&mut config);

 //   test_global_guard(&mut config);

	test_array(&mut config);

//	test_hash_table(&mut config);

//	test_queue(&mut config);
}


fn test_ptr(config: &mut Config) {
    // ----------- Global Pointer's part -------------
    #[derive(Debug, Clone)]
    struct HE {
        key: i64,
        value: i64,
        other: i64,
    }

    if config.rank == 0 { println!("------------Global Pointer's test------------\n"); }

    let mut ptr: Vec<GlobalPointer<HE>> = Vec::new();
    ptr.resize(config.rankn, GlobalPointer::null());
    comm::barrier();
    ptr[config.rank] = GlobalPointer::init(config, 1);
    comm::barrier();
    for i in 0..config.rankn {
        comm::broadcast(&mut ptr[i], i);
    }
    comm::barrier();

    for i in 0 .. 100 {
        for j in 0 .. config.rankn {
            let entry = HE {
                key: config.rank as i64,
                value: 11 * config.rank as i64,
                other: 12132
            };
            ptr[j].rput(entry);
            comm::barrier();
        }
    }
    comm::barrier();

    for i in 0 .. 100 {
        for j in 0 .. config.rankn {
            let entry = ptr[j].rget();
            println!("{}: ({}, {})", i, entry.key, entry.value);
            comm::barrier();
        }
    }
    comm::barrier();

}

fn test_global_pointer(config: &mut Config) {
	// ----------- Global Pointer's part -------------
    if config.rank == 0 { println!("------------Global Pointer's test------------\n"); }

    let mut ptr1 = GlobalPointer::null();
    if config.rank == 0 {
        let rankn = config.rankn;
        ptr1 = GlobalPointer::init(config, rankn);
    }
    comm::broadcast(&mut ptr1, 0);

    // test rput, rget
    (ptr1 + config.rank as isize).rput(config.rank as i32);
    comm::barrier();

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

    comm::barrier();
    println!("barrier1, rank{}!", config.rank);
    comm::barrier();

    if config.rank == 1 {
        println!("Rank 1 Sees: ");
        for i in 0..config.rankn {
            value = (ptr1 + i as isize).rget();
            println!("{}: {}", i, value);
        }
    }
    comm::barrier();

    // test idx_rget, idx_rput
    ptr1.idx_rput(config.rank as isize, 2*config.rank as i32);
    comm::barrier();

    let mut value;
    if config.rank == 1 {
    	println!("test idx_rget, idx_rput");
        println!("Rank 1 Sees: ");
        for i in 0..config.rankn {
            value = ptr1.idx_rget(i as isize);
            println!("{}: {}", i, value);
        }
    }
    comm::barrier();

    // test arput, arget
    let mut ptr2 = GlobalPointer::null();
    if config.rank == 0 {
        ptr2 = GlobalPointer::init(config, 6);
        let values = vec![0, 1, 2, 3, 4, 5];
        ptr2.arput(&values);
    }
    comm::broadcast(&mut ptr2, 0);
    if config.rank == 1 {
        println!("test arget, arput");
        let values = ptr2.arget(6);
        println!("Rank{}: arget {:?}", config.rank ,values);
    }
}

fn test_array(config: &mut Config) {
    // ----------- array's part ------------
    if config.rank == 0 { println!("\n\n------------Array's test------------\n"); }
    let rankn = config.rankn;
    let size_arr = 1024;
    let mut arr = Array::<i64>::init(config, size_arr);
    //arr.write(('a' as u8 + config.rank as u8) as char, config.rank);
    //arr.write(0 as i64, config.rank);
    for i in 0..size_arr {
        arr.write(0 as i64, i);
    }
    // comm::long_atomic_fetch(&mut ptr);
    comm::barrier();
    //println!("here1");
    let mut time1: time::Tm = time::now();
    let mut time_res: time::Duration;
    let mut time2: time::Tm;
    //let workload = 131072;
    let workload = 131072*rankn;
    //let size_arr = 1024;
    //let iters = (workload/rankn)/size_arr;
    let iters = workload/size_arr;
    if config.rank == 0 {
        time1 = time::now();
    }

    for i in 0..iters {
        for j in 0..size_arr {
            let mut ptr = arr.get_ptr(j);
            comm::long_atomic_fetch_add(&mut ptr, 1 as i64);
        }
    }
    comm::barrier();
    //println!("here2");
    if config.rank == 0 {
        time2 = time::now();
        time_res = time2 - time1;
        //println!("time is {:?}", time_res);
        for i in 0..size_arr {
            println!("{}: {}", i, arr.read(i));
        }
	println!("time is {:?}", time_res);
    }
}

fn test_hash_table(config: &mut Config) {
    // ----------- HashTable's part ------------
    if config.rank == 0 { println!("\n\n------------HashTable's test------------\n"); }

    let mut hash_table = HashTable::<usize, char>::new(config, 1024);

    let key: usize = 0;
    let value  = [char::from('a' as u8), char::from('A' as u8)];
//    let key: usize = config.rank;
//    let value  = [char::from('a' as u8 + config.rank as u8), char::from('A' as u8 + config.rank as u8)];
    let mut success = false;

    // Testing for Updating like "hash_table[key] = value"
    for _ in 0..5 {
        for i in 0..2 {
            success = hash_table.insert(&key, &value[i]);
            Config::barrier();
            println!("key is {}, val is {}, insert success = {} by rank {}", key, value[i], success, shmemx::my_pe());
            Config::barrier();
        }
    }

    comm::barrier();

    let mut res: char = '\0';
    for key in 0..(config.rankn + 1) {
        success = hash_table.find(&key, &mut res);
        if success {
            println!("key is {}, find value {:?} by rank {}", key, res, shmemx::my_pe());
        } else {
            println!("key is {}, find nothing by rank {}", key, shmemx::my_pe());
        }
    }
}

fn test_queue(config: &mut Config) {
    // ----------- Queue's part ------------
    comm::barrier();
    if config.rank == 0 { println!("\n\n------------Queue's test------------\n"); }
    let rankn = config.rankn;
    // ----------- Queue's part ------------
    comm::barrier();
//    if config.rank == 0 { println!("\n\n------------Queue's test------------\n"); }
    let mut queue = Queue::<char>::new(config, 2000);
    for i in 0..10 {
        queue.add(('a' as u8 + i as u8 + config.rank as u8) as char);
    }
    comm::barrier();

    if config.rank == 0 {
        let len = queue.len();
        for i in 0..len {
            let f = queue.remove();
            match f {
                Ok(data) => println!("index: {} value: {}", i, data),
                Err(err) => println!("{}", err),
            }

        }
    }
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
    comm::barrier();

    // text mutex
    let step = 100000;
    for i in 0..step {
    	let value = guard1.lock();
    	let t = value.rget();
    	value.rput(t + 1);
    }
    comm::barrier();

    if config.rank == 0 {
    	let value = guard1.lock();
    	let t = value.rget();
    	assert_eq!(t, step * config.rankn);
    	println!("Global Guard's test: pass! step: {}", step);
    }
}

fn test_shmem_atomic(config: &mut Config) {
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

    let value = guard1.lock();
    println!("this message should only be printed once");

    let result = guard1.test_lock();
    match result {
    	Ok(value) => println!("Get the lock again!"),
    	Err(error) => println!("That's right! It should not be able to get the lock!"),
    };
}
