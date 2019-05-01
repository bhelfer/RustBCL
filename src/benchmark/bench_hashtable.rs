use global_pointer::GlobalPointer;
use hash_table::HashTable;
use config::Config;
use std::time::{SystemTime, Duration};
use std::vec::Vec;
use comm;
use rand::rngs::StdRng;
use lib_bcl::shmemx;
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;
use std::env;

/// hash_table benchmarks
pub fn benchmark_hash_table(config: &mut Config) {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 { return Err("not enough arguments"); }

    let rankn: i64 = config.rankn as i64;
    let rank: i64 = config.rank as i64;

    let n: i64 = args[1].clone().parse().unwrap();
    let m: i64 = n / 2;

    let mut hash_table_lfz: HashTable<i64, i64> = HashTable::new(config, (n * rankn * 2) as usize);

    let mut keys: Vec<i64> = (-m .. m).collect();
    let mut values: Vec<i64> = (-m .. m).collect();
    keys.shuffle(&mut thread_rng());
    values.shuffle(&mut thread_rng());

    comm::barrier();

    let mut rng: StdRng = SeedableRng::from_seed([233; 32]);

    comm::barrier();
    let insert_start = SystemTime::now();
    for i in -m .. m {
        // all PE
        let key = keys[i];
        let value = values[i];
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
    for i in -m .. m {
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
