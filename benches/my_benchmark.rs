#![allow(unused)]
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
use lib_bcl::array::Array;

use hash_table::HashTable;
use self::rand::{Rng, SeedableRng, StdRng};
use global_pointer::GlobalPointer;

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
            match f {
                Ok(data) => println!("index: {} value: {}", i, data),
                Err(err) => println!("{}", err),
            }
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

fn same_entry_test() {

    let mut config = Config::init(32);
    let rankn: i64 = config.rankn as i64;
    let rank: i64 = config.rank as i64;

    let n: i64 = 10;
    let m: i64 = 10;

    let mut hash_table_ref: HashMap<i64, i64> = HashMap::new();
    let mut hash_table_lfz: HashTable<i64, i64> = HashTable::new(&mut config, (n*5) as usize);

    let mut k_ptr: GlobalPointer<i64> = GlobalPointer::null();
    let mut v_ptr: GlobalPointer<i64> = GlobalPointer::null();
    if rank == 0 {
        k_ptr = config.alloc::<i64>(1);
        v_ptr = config.alloc::<i64>(1);
    }
    comm::barrier();

    comm::broadcast(&mut k_ptr, 0);
    comm::broadcast(&mut v_ptr, 0);
    comm::barrier();

    let mut rng: StdRng = SeedableRng::from_seed([233; 32]);

    for i in 0 .. n {
        if rank == 0 {
            k_ptr.rput(rng.gen_range(-m, m));
            v_ptr.rput(rng.gen_range(-m, m));
        }
        comm::barrier();

        let key = k_ptr.rget();
        let value = v_ptr.rget();
        comm::barrier();

        // all PE
        let success = hash_table_lfz.insert(&key, &value);
        hash_table_ref.insert(key.clone(), value.clone());

        if success == false {
            panic!("HashTable({}) Agh! insertion failed", shmemx::my_pe());
        }

        comm::barrier();
    }

    comm::barrier();
    println!("HashTable({}) Done with insert!", shmemx::my_pe());
    comm::barrier();

    comm::barrier();

    for i in -m .. m {
        if (rank - i) % rankn == 0 {
            let v_ref = hash_table_ref.get(&i);
            let v_ref = match v_ref {
                Some(&v) => v,
                None => std::i64::MAX,
            };

            let mut v_lfz: i64 = 0;
            let mut success: bool = false;
            success = hash_table_lfz.find(&i, &mut v_lfz);

            if !success {
                v_lfz = std::i64::MAX;
            }

            println!("iter_find({}) {}, (v_ref, v_lfz) = ({}, {})", rank, i, v_ref, v_lfz);
            assert_eq!(v_ref, v_lfz);
        }

        comm::barrier();
    }

    comm::barrier();
}
fn distributed_array() {
    let mut config = Config::init(1);
    let rankn = config.rankn;
    let mut arr = Array::<i64>::init(&mut config, rankn);
    arr.write(0 as i64, config.rank);
    comm::barrier();
    let mut ptr = arr.get_ptr(0);
    for i in 0..1000 {
        comm::long_atomic_fetch_add(&mut ptr, 1 as i64);
    }
    comm::barrier();
    if config.rank == 0 {
        for i in 0..rankn {
            println!("{}: {}", i, arr.read(i));
        }
    }
}
fn original_array() {
    let mut config = Config::init(1);
    let rankn = config.rankn;
    const SIZE: usize = 100;
   // let mut array: [i64; size] = [0; size];
   unsafe{
    let mut arr: [i64; SIZE] = std::mem::uninitialized();
    for item in &mut arr[..] {
        std::ptr::write(item, 0);
    }
    for i in 0..1000 {
        std::ptr::write(&mut arr[0], arr[0] + 1);
    }
    for i in 0..100 {
        let x = std::ptr::read(& arr[i]);
        println!("{}", x);
    }
    }
}
fn criterion_benchmark(c: &mut Criterion) {
//    c.bench_function("same_entry test", |b| b.iter(|| same_entry_test()));
//    c.bench_function("Distributed queue test", |b| b.iter(|| distributed_queue()));
//    c.bench_function("Original queue test", |b|b.iter(||original_queue()));
    c.bench_function("Distributed array test", |b| b.iter(|| distributed_array()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
