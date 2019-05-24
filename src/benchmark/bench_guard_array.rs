use base::config::Config;
use backend::comm;
use containers::guard_array::GuardArray;

use std::time::{SystemTime};

use rand::Rng;
use benchmark::tools::duration_to_nano;

pub fn benchmark_guard_array(config: &mut Config, total_workload: usize, label: &str) {
    let array_size = 1024;
    let local_workload = (total_workload + config.rankn - 1) / config.rankn;

    let mut rng = rand::thread_rng();
    let mut garr = GuardArray::<i32>::init(config, array_size);

    // Initialize all values to 0
    if config.rank == 0 {
        for idx in 0..array_size {
            garr.write(0, idx);
        }
    }

    // Benchmark
    comm::barrier();
    let start = SystemTime::now();
    let mut ave_time1 = 0;
    let mut ave_time2 = 0;
    let mut ave_time3 = 0;

    for i in 0u128..local_workload as u128 {
        let idx = rng.gen_range(0, array_size);

        let start1 = SystemTime::now();
        let mut gval = garr.lock(idx);
        let time1 = duration_to_nano(&SystemTime::now().duration_since(start1).unwrap());
        ave_time1 = (time1 + ave_time1 * i) / (i + 1);

        let start2 = SystemTime::now();
        let t = gval.rget();
        let time2 = duration_to_nano(&SystemTime::now().duration_since(start2).unwrap());
        ave_time2 = (time2 + ave_time2 * i) / (i + 1);

        let start3 = SystemTime::now();
        gval.rput(t + 1);
        let time3 = duration_to_nano(&SystemTime::now().duration_since(start3).unwrap());
        ave_time3 = (time3 + ave_time3 * i) / (i + 1);
    }
    comm::barrier();
    let duration = SystemTime::now().duration_since(start)
        .expect("SystemTime::duration_since failed");
    let nanos = duration_to_nano(&duration);
    if config.rank == 0 {
        println!("GuardArray {}: {}, {}; {}, ({}, {}, {})", label, config.rankn, nanos,
                 local_workload, ave_time1, ave_time2, ave_time3);
    }
}