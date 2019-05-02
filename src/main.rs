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
use containers::{array::Array, hash_table::HashTable, queue::Queue};
use containers::guard_array::{GuardArray, GlobalGuardVec};
use benchmark::{bench_global_guard, bench_global_pointer, bench_shmem, bench_hashtable, bench_sample_sort, bench_1d_fft};
use backend::{comm, shmemx};

use self::rand::{Rng, StdRng, SeedableRng};
use std::collections::HashMap;
use std::mem::size_of;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use self::num::complex::{Complex, Complex32, Complex64};

fn main() {
    let mut config = Config::init(32);
    let rankn = config.rankn;

    bench_1d_fft::benchmark_1d_fft(&mut config);
//    bench_sample_sort::benchmark_sample_sort(&mut config);
//    strong_scaling_queue(&mut config);

//    test_ptr(&mut config);
//
//    test_global_pointer(&mut config);
//
//    test_shmem_atomic(&mut config);
//
//    test_global_guard(&mut config);
//
//    test_array(&mut config);
//
//    test_hash_table(&mut config);

//    test_queue(&mut config);


//    test_global_guard_vec(&mut config);

//    test_guard_array(&mut config);

//    benchmark_guard_array(&mut config);

//    bench_global_guard::benchmark_global_guard(&mut config);
//    bench_global_pointer::benchmark_global_pointer_remote(&mut config);
//    bench_global_pointer::benchmark_global_pointer_local(&mut config);
//    bench_global_pointer::benchmark_global_pointer_local_raw(&mut config);
//    bench_shmem::benchmark_shmem(&mut config);
//    bench_hashtable::benchmark_hash_table(&mut config);
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
    if config.rank == 1 {
        println!("Rank 1 Sees: ");
        for i in 0..config.rankn {
            value = (ptr1 + i as isize).rget();
            println!("{}: {}", i, value);
        }
    }
    comm::barrier();

    if config.rank == 0 {
        let p1 = ptr1.local();
        let p_slice = unsafe { std::slice::from_raw_parts(p1, config.rankn) };
        println!("Rank 0 Sees: ");
        for i in 0..config.rankn {
            value = p_slice[i];
            println!("{}: {}", i, value);
        }
    }
    comm::barrier();

    // test idx_rget, idx_rput
    ptr1.idx_rput(config.rank as isize, 2 * config.rank as i32);
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
        println!("Rank{}: arget {:?}", config.rank, values);
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
    let workload = 131072 * rankn;
    //let size_arr = 1024;
    //let iters = (workload/rankn)/size_arr;
    let iters = workload / size_arr;
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

fn weak_scaling_queue(config: &mut Config) {
    if config.rank == 0 { println!("\n------------Queue's weak scaling------------\n"); }
    let rankn = config.rankn;
    comm::barrier();
    let mut queue = Queue::<char>::new(config, 131080 * rankn);
    comm::barrier();
    let start = SystemTime::now();
    for _ in 0..131072 {
        queue.push(('a' as u8 + config.rank as u8) as char);
    }
    comm::barrier();
    let since_the_epoch = SystemTime::now().duration_since(start).expect("SystemTime::duration_since failed");
    if config.rank == 0 { println!("Insert time: {:?}, starting removing.", since_the_epoch); }

    comm::barrier();
    let start = SystemTime::now();
    for i in 0..131072 {
        let f = queue.pop();
    }
    let since_the_epoch = SystemTime::now().duration_since(start).expect("SystemTime::duration_since failed");
    if config.rank == 0 { println!("Removing time: {:?}.", since_the_epoch); }
//    comm::barrier();
//    if config.rank == 0 {
//        println!("Finished inserting!");
//        let mut count_vector = vec![0; rankn];
//        let len = queue.len();
//        println!("The length of the queue is {}.", len);
//        for i in 0..len {
//            let f = queue.remove();
//            match f {
//                Ok(data) => {
//                    let idx = (data as u32 - 'a' as u32) as usize;
//                    count_vector[idx] += 1;
//                }
//                Err(err) => println!("{}", err),
//            }
//        }
//        for i in 0..rankn {
//            println!("Data: {}, count: {}", ('a' as u8 + i as u8) as char, count_vector[i]);
//        }
//    }
}

fn strong_scaling_queue(config: &mut Config) {
    if config.rank == 0 { println!("\n------------Queue's strong scaling------------\n"); }
    let rankn = config.rankn;
    comm::barrier();
    let mut queue = Queue::<char>::new(config, 300000);
    let local_length = (131072 + rankn - 1) / rankn;
    comm::barrier();
    let start = SystemTime::now();
    for _ in 0..local_length {
        queue.push(('a' as u8 + config.rank as u8) as char);
    }
    comm::barrier();
    let since_the_epoch = SystemTime::now().duration_since(start).expect("SystemTime::duration_since failed");
    if config.rank == 0 { println!("Insert time: {:?}, start removing.", since_the_epoch); }
    if config.rank == 0 {println!("Length of queue: {}", queue.len());}
    comm::barrier();
    let start = SystemTime::now();
    for _ in 0..local_length {
        let f = queue.pop();
        match f {
            Err(error) => println!("{}", error),
            Ok(data) => (),
        }
    }
    let since_the_epoch = SystemTime::now().duration_since(start).expect("SystemTime::duration_since failed");
    if config.rank == 0 { println!("Removing time: {:?}.", since_the_epoch); }
    if config.rank == 0 {println!("Length of queue: {}", queue.len());}
}

fn test_global_guard(config: &mut Config) {
    // ----------- Global Guard's part -------------
    if config.rank == 0 { println!("------------Global Guard's test------------\n"); }

    let step = 100000;
    let mut guard1 = GlobalGuard::null();
    if config.rank == 0 {
        guard1 = GlobalGuard::init(config);
    }
    comm::broadcast(&mut guard1, 0);

    if config.rank == 0 {
        let value = guard1.lock();
        value.rput(0);
    }
    comm::barrier();

    // text mutex
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

/// hash_table benchmarks
pub fn hash_table_benchmark(scale: i64) {
    let mut config = Config::init(512);
    let rankn: i64 = config.rankn as i64;
    let rank: i64 = config.rank as i64;

    let n: i64 = scale;
    let m: i64 = n / 2;

    let mut hash_table_lfz: HashTable<i64, i64> = HashTable::new(&mut config, (n * 2) as usize);
    comm::barrier();

    let mut rng: StdRng = SeedableRng::from_seed([233; 32]);

    comm::barrier();
    let insert_start = SystemTime::now();
    for i in -m..m {
        // all PE
        let key = rng.gen_range(-m, m);
        let value = rng.gen_range(-m, m);
        let success = hash_table_lfz.insert(&key, &value);
        if success == false {
            panic!("HashTable({}) Agh! insertion failed", shmemx::my_pe());
        }
        comm::barrier();
    }
    comm::barrier();
    let insert_time = SystemTime::now().duration_since(insert_start)
        .expect("SystemTime::duration_since failed");

    // println!("HashTable({}) Done with insert!", shmemx::my_pe());

    comm::barrier();
    let find_start = SystemTime::now();
    for i in -m..m {
        let mut v_lfz: i64 = 0;
        let mut success: bool = false;
        success = hash_table_lfz.find(&i, &mut v_lfz);
        comm::barrier();
    }
    comm::barrier();
    let find_time = SystemTime::now().duration_since(find_start)
        .expect("SystemTime::duration_since failed");
    if shmemx::my_pe() == 0 {
        println!("(insert_time, find_time) = ({:?}, {:?})", insert_time, find_time);
    }
}

fn test_global_guard_vec(config: &mut Config) {
    // ----------- GlobalGuardVec's part -------------
    if config.rank == 0 { println!("------------GlobalGuardVec's test------------\n"); }

    // Initialize a guard array
    let array_size = 8;
    let step = 1000;
    let mut guard_vec1 = GlobalGuardVec::<i16>::null();

    // Initialize all values to 0
    if config.rank == 0 {
        guard_vec1 = GlobalGuardVec::init(config, array_size);
        for idx in 0..array_size {
            let value = guard_vec1.lock(idx);
            value.rput(0);
        }
    }
    comm::broadcast(&mut guard_vec1, 0);

    for _ in 0..step {
        for idx in 0..array_size {
            let gval = guard_vec1.lock(idx);
            gval.rput(gval.rget() + 1);
        }
    }
    comm::barrier();

    if config.rank == 0 {
        for idx in 0..array_size {
            let value = guard_vec1.lock(idx);
            assert_eq!(value.rget(), step * config.rankn as i16, "error at idx: {}", idx);
        }
        println!("GlobalGuardVec test passed!");
    }
}

fn test_guard_array(config: &mut Config) {
    // ----------- GuardArray's part -------------
    if config.rank == 0 { println!("------------Guard Array's test------------\n"); }

    // Initialize a guard array
    let array_size = 128;
    let step = 1000;
    let mut garr = GuardArray::<i32>::init(config, array_size);

    // Initialize all values to 0
    if config.rank == 0 {
        for idx in 0..array_size {
            garr.write(0, idx);
        }
    }
    comm::barrier();

    for _ in 0..step {
        for idx in 0..array_size {
            let gval = garr.lock(idx);
            gval.rput(gval.rget() + 1);
        }
    }
    comm::barrier();

    if config.rank == 0 {
        for idx in 0..array_size {
            assert_eq!(garr.read(idx), step * config.rankn as i32, "error at idx: {}", idx);
        }
        println!("Guard Array test passed!");
    }
}

fn benchmark_guard_array(config: &mut Config) {
    let array_size = 1024;
//    let total_workload = 131072; // strong scaling
    let total_workload = 131072 * config.rankn; // weak scaling
    let local_workload = (total_workload + config.rankn - 1) / config.rankn;

    let mut rng = rand::thread_rng();
    let mut garr = GuardArray::<i32>::init(config, array_size);

    // Initialize all values to 0
    if config.rank == 0 {
        for idx in 0..array_size {
            garr.write(0, idx);
        }
    }
    comm::barrier();
    let start = SystemTime::now();

    for _ in 0..local_workload {
        let idx = rng.gen_range(0, array_size);
        let gval = garr.lock(idx);
        gval.rput(gval.rget() + 1);
    }
    comm::barrier();
    let since_the_epoch = SystemTime::now().duration_since(start)
        .expect("SystemTime::duration_since failed");
    if config.rank == 0 {
        println!("rank num: {}; table size: {}; total workload: {}; time: {:?}",
                 config.rankn, array_size, total_workload, since_the_epoch);
    }
}
