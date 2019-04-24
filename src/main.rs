#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

extern crate rand;

mod shmemx;
mod global_pointer;
mod config;
mod comm;
mod array;
mod hash_table;

mod queue;
use config::Config;
use global_pointer::GlobalPointer;
use array::Array;
use hash_table::HashTable;
use queue::Queue;

use self::rand::{Rng, SeedableRng, StdRng};
use std::collections::HashMap;

fn main() {

    let mut config = Config::init(1);
    let rankn = config.rankn;

    println!("Hello, fam! I'm {} out of {}", config.rank, config.rankn);

    comm::barrier();

    test_hash_table(&mut config);
}

fn test_hash_table(config: &mut Config) {
    // ----------- HashTable's part ------------
    if config.rank == 0 { println!("\n\n------------HashTable's test------------\n"); }

    let mut hash_table = HashTable::<usize, char>::new(config, 1024);

    hash_table.print(config);
    if config.rank == 0 {
        hash_table.insert(&12, &'a');
    }

    comm::barrier();
    println!("Rank {} after barrier", config.rank);
    comm::barrier();
    /*
    let result = hash_table.insert(&config.rank, &(config.rank as u8 as char));

    println!("Rank {} got rv {}", config.rank, result);
    */

    /*
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
    */
}
