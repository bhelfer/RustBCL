#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

extern crate is_sorted;
extern crate quickersort;

use base::{global_pointer::GlobalPointer, config::Config};
use backend::comm;

use std::time::{SystemTime, Duration};
use std::vec::Vec;
use std::env;
use rand::{rngs::StdRng, Rng, thread_rng, SeedableRng, ChaChaRng};
use is_sorted::IsSorted;

pub fn benchmark_sample_sort(config: &mut Config) {

    let args: Vec<String> = env::args().collect();
    let n: usize;

    if args.len() <= 1 {
        println!("not enough arguments\nUse default argument n = 1000");
        n = 1000;
    } else {
        n = args[1].clone().parse().unwrap();
    }

    let rankn: usize = config.rankn as usize;
    let rank: usize = config.rank as usize;

    let size: usize = n * rankn;

    let mut data: GlobalPointer<u32> = GlobalPointer::init(config, size);
    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);

    if rank == 0 {
        for i in 0 .. size {
            data.idx_rput(i as isize, rng.gen_range(0, std::u32::MAX));
//            data.idx_rput(i as isize, i as u32);
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
    let mut serial_local = local[rank].arget(n);
    quickersort::sort(&mut serial_local[..]);
    comm::barrier();

    assert_eq!(IsSorted::is_sorted(&mut serial_local.iter()), true);
    comm::barrier();

    //
}