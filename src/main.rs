#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

extern crate rand;
extern crate statistical;
extern crate time;
extern crate is_sorted;
extern crate num;

pub mod backend;
pub mod base;
pub mod containers;
pub mod benchmark;

use base::{global_pointer::{Bclable, GlobalPointer}, global_guard::GlobalGuard, config::Config};
use containers::{
    array::Array,
    hash_table::HashTable,
    queue::Queue,
    guard_array::{GuardArray, GlobalGuardVec}
};
use benchmark::{
    bench_global_guard,
    bench_global_pointer,
    bench_shmem,
    bench_hashtable,
    bench_sample_sort,
    bench_fft,
    bench_guard_array,
    bench_queue
};
use backend::{comm, shmemx};

use self::rand::{Rng, StdRng, SeedableRng};
use std::collections::HashMap;
use std::mem::size_of;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use self::num::complex::{Complex, Complex32, Complex64};

fn main() {
    let mut config = Config::init(2048);
    let rankn = config.rankn;

//    test_ptr(&mut config);
//    test_global_pointer(&mut config);
//    test_shmem_atomic(&mut config);
//    test_global_guard(&mut config);
//    test_array(&mut config);
//    test_hash_table(&mut config);
//    test_queue(&mut config);
//    test_global_guard_vec(&mut config);
//    test_guard_array(&mut config);

//    bench_global_guard::benchmark_global_guard_remote(&mut config);
//    bench_global_guard::benchmark_global_guard_local(&mut config);
//    bench_global_pointer::benchmark_global_pointer_remote(&mut config);
//    bench_global_pointer::benchmark_global_pointer_local(&mut config);
//    bench_global_pointer::benchmark_global_pointer_local_raw(&mut config);
//    bench_shmem::benchmark_shmem_atomic_cas(&mut config);
//    bench_shmem::benchmark_shmem_atomic_fetch_put(&mut config);
//    bench_hashtable::benchmark_hash_table(&mut config);
//    bench_sample_sort::benchmark_sample_sort(&mut config);
//    bench_fft::benchmark_fft(&mut config);
//    bench_sample_sort::benchmark_sample_sort(&mut config);

//    let workload = 131072;
//    let label = "strong sclaing";
//    benchmark(&mut config, workload, label);

    let workload = 131072 * config.rankn;
    let label = "weak sclaing";
    benchmark(&mut config, workload, label);
}

fn benchmark(config: &mut Config, workload: usize, label: &str) {
    bench_guard_array::benchmark_guard_array(config, workload, label);
//    bench_queue::benchmark_queue(config, workload, label);
//    bench_hashtable::benchmark_hash_table(config, workload, label);
}


fn test_ptr(config: &mut Config) {
    // ----------- Global Pointer's part -------------
    #[derive(Debug, Clone, Copy)]
    struct HE {
        key: i64,
        value: i64,
        other: i64,
    }
    impl Bclable for HE {}

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

    for i in 0..100 {
        for j in 0..config.rankn {
            let entry = HE {
                key: config.rank as i64,
                value: 11 * config.rank as i64,
                other: 12132,
            };
            ptr[j].rput(entry);
            comm::barrier();
        }
    }
    comm::barrier();

    for i in 0..100 {
        for j in 0..config.rankn {
            let entry = ptr[j].rget();
            println!("{}: ({}, {})", i, entry.key, entry.value);
            comm::barrier();
        }
    }
    comm::barrier();
}

fn test_global_pointer(config: &mut Config) {
    // ----------- Global Pointer's part -------------
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
    if config.rank == 1 {
        for i in 0..config.rankn {
            value = (ptr1 + i as isize).rget();
            assert_eq!(i as i32, value, "fail at rput, rget");
        }
    }
    comm::barrier();

    if config.rank == 0 {
        let p1 = ptr1.local();
        let p_slice = unsafe { std::slice::from_raw_parts(p1, config.rankn) };
        for i in 0..config.rankn {
            value = p_slice[i];
            assert_eq!(i as i32, value, "fail at test local");
        }
    }
    comm::barrier();

    // test idx_rget, idx_rput
    ptr1.idx_rput(config.rank as isize, 2 * config.rank as i32 + 1);
    comm::barrier();

    let mut value;
    if config.rank == 1 {
        for i in 0..config.rankn {
            value = ptr1.idx_rget(i as isize);
            assert_eq!(2*i as i32 + 1, value, "fail at idx_rget, idx_rput");
//            println!("{}, {}", i, value);
        }
    }
    comm::barrier();

    // test arput, arget
    let values = vec![0, 1, 2, 3, 4, 5];
    let mut ptr2 = GlobalPointer::null();
    if config.rank == 0 {
        ptr2 = GlobalPointer::init(config, 6);
        ptr2.arput(&values);
    }
    comm::broadcast(&mut ptr2, 0);
    if config.rank == 1 {
        let values2 = ptr2.arget(6);
        assert_eq!(values, values2, "fail at arget, arput");
    }
    if config.rank == 0 { println!("Global Pointer correctness test pass!   "); }
}


fn test_array(config: &mut Config) {

    let rankn: i32 = config.rankn as i32;
    let rank: i32 = config.rank as i32;
    let array_size = 1024;
    let total_workload = 131072; // strong scaling
   // let total_workload = 131072 * config.rankn; // weak scaling
    let local_workload = (total_workload + config.rankn - 1) / config.rankn;
    let mut arr = Array::<i64>::init(config, array_size);
    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);
    if config.rank == 0 {
        for i in 0.. array_size {
            arr.write(0 as i64, i as usize);
        }
    }
    comm::barrier();
    let mut time1: time::Tm = time::now();
    let mut time_res: time::Duration;
    let mut time2: time::Tm;
    if config.rank == 0 {
        time1 = time::now();
    }
    for i in 0..local_workload/array_size{
//        let mut ptr = arr.get_ptr(rng.gen_range(0, array_size as i32) as usize);
//        comm::long_atomic_fetch_add(&mut ptr, 1 as i64);
	for j in 0..array_size{  
		let mut ptr = arr.get_ptr(j);
        	comm::long_atomic_fetch_add(&mut ptr, 1 as i64);
    	}
    }
    comm::barrier();
    if config.rank == 0 {
        time2 = time::now();
        time_res = time2 - time1;
        for i in 0..array_size {
            println!("{}: {}", i, arr.read(i));
        }
        println!("time is {:?}", time_res);
    }
}
fn test_queue(config: &mut Config) {
    if config.rank == 0 { println!("\n------------Queue's strong scaling------------\n"); }
    let rankn = config.rankn;
    comm::barrier();
    let mut queue = Queue::<char>::new(config, 30000);
    let local_length = 500;
    if config.rank == 0 { println!("Local length is {}.", local_length); }
    if config.rank == 0 { println!("Before inserting, length of the queue is {}, is empty: {}.", queue.len(), queue.is_empty()); }
    comm::barrier();
    for _ in 0..local_length {
        queue.push(('a' as u8 + config.rank as u8) as char);
    }
    comm::barrier();
    if config.rank == 0 { println!("After insertion, length of the queue is {}, is empty: {}.", queue.len(), queue.is_empty()); }
    for _ in 0..local_length {
        let f = queue.pop();
        match f {
            Err(error) => {
                println!("{}", error);
                break;
            },
            Ok(result) => (),
        }
    }
    comm::barrier();
    if config.rank == 0 {
        println!("After popping, length of the queue is {}, is empty: {}.", queue.len(), queue.is_empty());
        queue.push('a');
        println!("Before clear, length of the queue is {}, is empty: {}.", queue.len(), queue.is_empty());
        queue.clear();
        println!("After clear, length of the queue is {}, is empty: {}.", queue.len(), queue.is_empty());
    }
}


fn test_hash_table(config: &mut Config) {
    // ----------- HashTable's part ------------
    if config.rank == 0 { println!("\n\n------------HashTable's test------------\n"); }

    let mut hash_table = HashTable::<usize, char>::new(config, 1024);

    let key: usize = 0;
    let value = [char::from('a' as u8), char::from('A' as u8)];
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