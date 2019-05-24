//#![allow(dead_code)]
//#![allow(unused)]
//#![allow(deprecated)]

use base::config::Config;
use backend::comm;
use containers::hash_table::HashTable;

use std::time::{SystemTime, Duration};
use rand::{rngs::StdRng, Rng, thread_rng, SeedableRng, seq::SliceRandom};
use benchmark::tools::duration_to_nano;

/// hash_table benchmarks
pub fn benchmark_hash_table(config: &mut Config, total_workload: usize, label: &str) {
    let local_workload = (total_workload + config.rankn - 1) / config.rankn;

    let mut hash_table_lfz: HashTable<i32, i32> = HashTable::new(config, (total_workload * 2) as usize);

    let mut rng: StdRng = SeedableRng::from_seed([config.rankn as u8; 32]);
    let mut keys: Vec<i32> = Vec::new();
    let mut values: Vec<i32> = Vec::new();
    for i in 0 .. local_workload {
        keys.push(rng.gen_range(std::i32::MIN, std::i32::MAX));
        values.push(rng.gen_range(std::i32::MIN, std::i32::MAX));
    }
    keys.shuffle(&mut thread_rng());
    values.shuffle(&mut thread_rng());

    comm::barrier();
    let insert_start = SystemTime::now();
    for i in 0 .. local_workload {
        // all PE
        let success = hash_table_lfz.insert(&keys[i as usize], &values[i as usize]);
        if success == false {
            panic!("HashTable({}) Agh! insertion failed", config.rank);
        }
    }
    comm::barrier();
    let insert_duration = SystemTime::now().duration_since(insert_start)
        .expect("SystemTime::duration_since failed");
    let insert_nanos = duration_to_nano(&insert_duration);

    comm::barrier();
    let find_start = SystemTime::now();
    for i in keys.iter() {
        let mut v_lfz: i32 = 0;
        let mut success: bool = false;
        success = hash_table_lfz.find(&i, &mut v_lfz);
    }
    comm::barrier();
    let find_duration = SystemTime::now().duration_since(find_start)
        .expect("SystemTime::duration_since failed");
    let find_nanos = duration_to_nano(&find_duration);

    if config.rank == 0 {
        println!("HashTable {}: {}, {}, {}", label, config.rankn, insert_nanos, find_nanos);
    }
}
