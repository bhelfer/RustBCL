use base::{config::Config, global_guard::GlobalGuard};
use backend::comm;

use std::time::{SystemTime, Duration};
use std::vec::Vec;
//use comm;
extern crate rand;
extern crate statistical;
use statistical;

fn duration_to_nano(duration: &Duration) -> u128 {
    let in_nanos = duration.as_secs() as u128 * 1000_000_000 +
            duration.subsec_nanos() as u128;
    in_nanos
}

pub fn benchmark_global_guard(config: &mut Config) {
    let iter = 100; // iter per step
    let step = 10000;

    // setup code
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

    // benchmark code
    let mut data = Vec::new();
    if config.rank == 1 {
        for _ in 0..step {
            let start = SystemTime::now();
            for _ in 0..iter {
                let value = guard1.lock();
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
        println!("Global Guard's Benchmark: mean: {:.2} nanos, std: {:.2} nanos", mean, standard_deviation);
    }
}