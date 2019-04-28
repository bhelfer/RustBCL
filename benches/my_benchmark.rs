#![allow(unused)]
#![allow(deprecated)]

#[macro_use]
extern crate criterion;

extern crate lib_bcl;
extern crate rand;

use criterion::Criterion;
use criterion::black_box;

use lib_bcl::hash_table;
use lib_bcl::config::Config;
use lib_bcl::global_pointer;
use lib_bcl::comm;
use lib_bcl::shmemx;
use lib_bcl::queue::Queue;
use std::collections::HashMap;
use std::collections::VecDeque;

use hash_table::HashTable;
use self::rand::{Rng, SeedableRng, StdRng};
use global_pointer::GlobalPointer;
use criterion::Benchmark;

fn distributed_queue() {
    let mut config = Config::init(16);
    let rankn = config.rankn;
    comm::barrier();
    let mut queue = Queue::<char>::new(&mut config, 2000);
    for i in 0..100 {
        queue.add((i as u8 + config.rank as u8) as char);
    }
    comm::barrier();

    if config.rank == 0 {
        let len = queue.len();
        for i in 0..len {
            let f = queue.remove();
//            match f {
//                Ok(data) => println!("index: {} value: {}", i, data),
//                Err(err) => println!("{}", err),
//            }
        }
    }
}

fn original_queue() {
    let mut queue:VecDeque<char> = VecDeque::with_capacity(200);
    for i in 0..100 {
        queue.push_back(('a' as u8 + i as u8) as char);
    }
    let len = queue.len();
    for i in 0..len {
        let f = queue.pop_front();
//        match f {
//            Some(data) => println!("index: {} value: {}", i, data),
//            None => println!("No data!"),
//        }
    }

}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench(
            "Default Group",
            Benchmark::new("Distributed queue test", |b| b.iter(|| distributed_queue()))
            .with_function("Original queue test", |b| b.iter(|| original_queue()))
            .sample_size(3)
    );
//    c.bench_function("Original queue test", |b|b.iter(||original_queue()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);