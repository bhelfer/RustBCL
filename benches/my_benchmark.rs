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
use criterion::ParameterizedBenchmark;
use std::rc::Rc;

fn distributed_queue(config: &mut Config) {
//    let mut config = Config::init(16);
    let rankn = config.rankn;
    comm::barrier();
    let mut queue = Queue::<char>::new(config, 2000);
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
    let n = 100000000;
    let mut queue:VecDeque<char> = VecDeque::with_capacity(n);
    for i in 0..n {
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
            "Default",
            Benchmark::new("Distributed queue test", move |b| b.iter(|| distributed_queue(&mut Config::init(16))))
            .with_function("Original queue test", |b| b.iter(|| original_queue()))
            .sample_size(3)
    );
//    c.bench_function("Original queue test", |b|b.iter(||original_queue()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);