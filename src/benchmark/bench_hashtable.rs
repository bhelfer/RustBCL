#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use base::{global_pointer::GlobalPointer, config::Config};
use backend::{comm, shmemx};
use containers::hash_table::HashTable;

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
//    println!("n, rankn, rank = ({}, {}, {})", n, rankn, rank);

    let mut hash_table_lfz: HashTable<i32, i32> = HashTable::new(config, (n * rankn * 2) as usize);

    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);
    let mut keys: Vec<i32> = Vec::new();
    let mut values: Vec<i32> = Vec::new();
    for i in 0 .. n {
        keys.push(rng.gen_range(std::i32::MIN, std::i32::MAX));
        values.push(rng.gen_range(std::i32::MIN, std::i32::MAX));
    }
    keys.shuffle(&mut thread_rng());
    values.shuffle(&mut thread_rng());

    comm::barrier();
    let insert_start = SystemTime::now();
    for i in 0 .. n {
        // all PE
        let success = hash_table_lfz.insert(&keys[i as usize], &values[i as usize]);
        if success == false {
            panic!("HashTable({}) Agh! insertion failed", shmemx::my_pe());
        }
//        comm::barrier();
    }
    comm::barrier();
    let insert_time = SystemTime::now().duration_since(insert_start)
        .expect("SystemTime::duration_since failed");

    // println!("HashTable({}) Done with insert!", shmemx::my_pe());

    comm::barrier();
    let find_start = SystemTime::now();
    for i in keys.iter() {
        let mut v_lfz: i32 = 0;
        let mut success: bool = false;
        success = hash_table_lfz.find(&i, &mut v_lfz);
//        comm::barrier();
    }
    comm::barrier();
    let find_time = SystemTime::now().duration_since(find_start)
        .expect("SystemTime::duration_since failed");
    if shmemx::my_pe() == 0 {
        println!("(insert_time, find_time) = ({:?}, {:?})", insert_time, find_time);
    }
}
