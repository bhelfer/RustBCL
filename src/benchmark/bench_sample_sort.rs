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

    let rankn: usize = config.rankn as usize;
    let rank: usize = config.rank as usize;

    let n: usize = args[1].clone().parse().unwrap();
    let size: usize = n * rankn;

    let mut data: GlobalPointer<u32> = GlobalPointer::init(config, size);
    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);

    if rank == 0 {
        for i in 0 .. size {
//            data.idx_rput(i as isize, rng.gen_range(0, std::u32::MAX));
            data.idx_rput(i as isize, i as u32);
        }
    }
    comm::barrier();

    // here start the sorting

    // scattering data
    let mut local: Vec<GlobalPointer<u32>> = Vec::new();
    local.resize(rankn as usize, GlobalPointer::null());
    local[rank] = GlobalPointer::init(config, n as usize);
    comm::barrier();

    comm::scatter(&mut local[rank], &mut data, 0, n);

    // local sorting


}