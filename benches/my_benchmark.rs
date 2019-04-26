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

use hash_table::HashTable;
use self::rand::{Rng, SeedableRng, StdRng};
use global_pointer::GlobalPointer;

fn test_queue() {
    let mut config = Config::init(1);
    let rankn = config.rankn;
    // ----------- Queue's part ------------
    comm::barrier();
    if config.rank == 0 { println!("\n\n------------Queue's test------------\n"); }
    let mut queue = Queue::<char>::new(&mut config, 10);
    queue.add(('a' as u8 + config.rank as u8) as char);
    comm::barrier();
    queue.add(('c' as u8 + config.rank as u8) as char);
    comm::barrier();

    if config.rank == 0 {
        let len = queue.len();
        println!("The length of the queue is {}.", len);
        println!("Peeking");
        {
            let t = queue.peek();
            match t {
                Ok(data) => println!("head value: {}", data),
                Err(err) => println!("{}", err),
            }
        }

        println!("Removing");
        for i in 0..len {
            let f = queue.remove();
            match f {
                Ok(data) => println!("index: {} value: {}", i, data),
                Err(err) => println!("{}", err),
            }

        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {

    c.bench_function("same_entry test", |b| b.iter(|| test_queue()));
//    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
