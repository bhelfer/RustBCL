#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use base::{global_pointer::GlobalPointer, config::Config};
use backend::comm;

use std::time::{SystemTime, Duration};
use std::vec::Vec;
use std::env;
use rand::{rngs::StdRng, Rng, thread_rng, SeedableRng, ChaChaRng};

pub fn benchmark_sample_sort(config: &mut Config) {

    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 { panic!("not enough arguments"); }

    let rankn: u32 = config.rankn as u32;
    let rank: u32 = config.rank as u32;

    let n: u32 = args[1].clone().parse().unwrap();
    let size: u32 = n * rankn;

    let mut data: Vec<i32> = Vec::new();
    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);
    for i in 0 .. size {
        data.push(rng.gen_range(0, std::i32::MAX));
    }

    // here start the sorting

    // scattering data
    let mut local: Vec<GlobalPointer<i32>> = Vec::new();
    local.resize(rankn as usize, GlobalPointer::null());
    local[rank] = GlobalPointer::init(config, n as usize);
    for i in 0 .. n {
        (local[rank] + i as isize).rput(data[rank * n + i]);
    }


}