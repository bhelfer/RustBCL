#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use base::{global_pointer::GlobalPointer, config::Config};
use backend::{comm, shmemx};
use containers::array::Array;

use std::time::{SystemTime, Duration};
use std::vec::Vec;
use std::env;

use rand::{rngs::StdRng, Rng, thread_rng, SeedableRng, ChaChaRng, seq::SliceRandom};

/// hash_table benchmarks
pub fn benchmark_hash_table(config: &mut Config) {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 { panic!("not enough arguments"); }

    let rankn: i32 = config.rankn as i32;
    let rank: i32 = config.rank as i32;
    
    let n: i32 = args[1].clone().parse().unwrap();
    let size_array = (n*rankn) as usize
    let workload = 131072 * rankn;
    let iters = workload / size_array;
//    println!("n, rankn, rank = ({}, {}, {})", n, rankn, rank);
    let mut arr = Array::<i64>::init(config, size_array);
    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);
    /*for i in 0..(n*rankn)  {
        arr.write(0 as i64, i);
    }*/
    for i in rank..(rank+n) {
        arr.write(0 as i64, i)
    }
    comm::barrier();
    let mut time1: time::Tm = time::now();
    let mut time_res: time::Duration;
    let mut time2: time::Tm;
    if config.rank == 0 {
        time1 = time::now();
    }
    for i in 0..iters {
        for j in 0..size_array {
            //let mut ptr = arr.get_ptr(j);
            let mut pts = arr.get_ptr(rng.gen_range(0, size_array as i64))
            comm::long_atomic_fetch_add(&mut ptr, 1 as i64);
        }
    }
    comm::barrier();
    if config.rank == 0 {
        time2 = time::now();
        time_res = time2 - time1;
        for i in 0..size_arr {
            println!("{}: {}", i, arr.read(i));
        }
        println!("time is {:?}", time_res);
    }
}
