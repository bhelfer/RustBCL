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
use std::time::Duration;

fn distributed_queue(mut config: Config) {
    let n = 1000000;

    let rankn = config.rankn;
    let mut queue = Queue::<char>::new(&mut config, rankn * n);
    for i in 0..n {
        queue.add((i as u8 + config.rank as u8) as char);
    }
    comm::barrier();

    if config.rank == 0 {
        let len = queue.len();
        for i in 0..len {
            let f = queue.remove();
        }
    }
    comm::barrier();
}

fn original_queue() {
    let n = 1000000;
    let mut queue:VecDeque<char> = VecDeque::with_capacity(n);
    for i in 0..n {
        queue.push_back(('a' as u8 + i as u8) as char);
    }
    let len = queue.len();
    for i in 0..len {
        let f = queue.pop_front();
    }
}

/// measurement_time(): Changes the default measurement time for benchmarks run with this runner.
/// sample_size(): Changes the default size of the sample for benchmarks run with this runner.

/// with_function(): add another tested function after Benchmark::new()

/// iter_with_large_drop(): In this case, the values returned by the benchmark are collected
/// into a Vec to be dropped after the measurement is complete.

fn criterion_benchmark(c: &mut Criterion) {
    c.bench(
            "Default",
            Benchmark::new(
                "Distributed queue test",
                |b| b.iter_with_large_drop(|| distributed_queue(Config::init(32)))
            )
            .with_function(
                "Original queue test",
                |b| b.iter_with_large_drop(|| original_queue())
            )
            .sample_size(2).measurement_time(Duration::from_secs(5))
    );
//    c.bench_function("Original queue test", |b|b.iter(||original_queue()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);