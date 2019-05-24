use base::{config::Config, global_guard::GlobalGuard};
use backend::comm;
use benchmark::tools::duration_to_nano;

use std::time::{SystemTime, Duration};
use std::vec::Vec;

use statistical;

pub fn benchmark_global_guard_local(config: &mut Config) {
    let iter = 100; // iter per step
    let step = 10000;

    // setup code
    let mut guard1 = GlobalGuard::null();
    if config.rank == 0 {
        guard1 = GlobalGuard::init(config);
    }
    comm::broadcast(&mut guard1, 0);

    if config.rank == 0 {
        let mut value = guard1.lock();
        value.rput(0);
    }
    comm::barrier();

    // benchmark code
    let mut data = Vec::new();
    if config.rank == 0 {
        for _ in 0..step {
            let start = SystemTime::now();
            for _ in 0..iter {
                let mut value = guard1.lock();
                let t = value.rget();
                value.rput(t + 1);
            }
            let duration = SystemTime::now().duration_since(start).unwrap();
            let nanos = duration_to_nano(&duration) / iter;
            data.push(nanos as f64);
        }
    }
    comm::barrier();

    if config.rank == 0 {
    	assert_eq!(guard1.lock().rget(), iter*step);
        let mean = statistical::mean(&data);
        let standard_deviation = statistical::standard_deviation(&data, None);
        println!("Global Guard(local)'s Benchmark: mean: {:.2} nanos, std: {:.2} nanos", mean, standard_deviation);
    }
}

pub fn benchmark_global_guard_remote(config: &mut Config) {
    let iter = 100; // iter per step
    let step = 10000;

    // setup code
    let mut guard1 = GlobalGuard::null();
    if config.rank == 0 {
        guard1 = GlobalGuard::init(config);
    }
    comm::broadcast(&mut guard1, 0);

    if config.rank == 0 {
        let mut value = guard1.lock();
        value.rput(0);
    }
    comm::barrier();

    // benchmark code
    let mut data = Vec::new();
    if config.rank == 1 {
        for _ in 0..step {
            let start = SystemTime::now();
            for _ in 0..iter {
                let mut value = guard1.lock();
                let t = value.rget();
                value.rput(t + 1);
            }
            let duration = SystemTime::now().duration_since(start).unwrap();
            let nanos = duration_to_nano(&duration) / iter;
            data.push(nanos as f64);
        }
    }
    comm::barrier();

    if config.rank == 1 {
    	assert_eq!(guard1.lock().rget(), iter*step);
        let mean = statistical::mean(&data);
        let standard_deviation = statistical::standard_deviation(&data, None);
        println!("Global Guard(remote)'s Benchmark: mean: {:.2} nanos, std: {:.2} nanos", mean, standard_deviation);
    }
}