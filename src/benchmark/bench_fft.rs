#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]
#![allow(non_snake_case)]

use num::complex::{Complex, Complex32, Complex64};
use rand::{rngs::StdRng, Rng, thread_rng, SeedableRng, ChaChaRng};

use base::{global_pointer::GlobalPointer, config::Config};
use backend::comm;

use containers::array;
use containers::array::Array;

use std::time::{SystemTime, Duration};
use std::vec::Vec;
use std::env;
use std::f64::consts::PI;
use std::mem;
use std::process::exit;

type Tp = f64;
type Cp = Complex<Tp>;

pub fn benchmark_fft(config: &mut Config) {

    let args: Vec<String> = env::args().collect();
    let mut N: usize;
    let min_size: usize = (1 << 3);
    // output debug info or not
    let mut DBG: bool = true;

    let rankn: usize = config.rankn as usize;
    let rank: usize = config.rank as usize;
    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);

    if (rankn & (rankn - 1)) != 0 {
        if rank == 0 { println!("only support rankn of 2's power now"); }
        return;
    }

    if args.len() <= 1 {
        N = min_size;
        if rank == 0 { println!("not enough arguments\nUse default argument N = {}", N); }
    } else {
        N = args[1].clone().parse().unwrap();
        N = (1 << ((N as f64).log2().ceil() as usize));
        if N < min_size { N = min_size; }
    }

    N *= rankn;
    if N >= 100 { DBG = false; }

    // double N for polynomial squaring
    let mut data = Array::init(config, 2 * N);
    comm::barrier();

    let n = data.local_size;
    let offset: usize = n * rank;

//    if rank == 0 { println!("n = {}, N = {}", n, N); }
    for i in 0 .. n {
        let idx = i + offset;

        let mut val = idx as f64;
        if idx >= N { val = 0.0; }

        data.write(Complex::new(val, 0.0), idx);
    }
    comm::barrier();

    /* debug */ if DBG { print_array(config, &data, n, "input"); }

    // here start the parallel fft

    // forward fft, dir = 1
    // just recording time for one fft
    comm::barrier();
    let start_time = SystemTime::now();

    fft_parallel(config, &mut data, 1);

    comm::barrier();
    let total_time = SystemTime::now().duration_since(start_time)
        .expect("SystemTime::duration_since failed");
    if rank == 0 { println!("total_time = {:?}", total_time); }

    comm::barrier();
    return;

    for i in 0 .. n {
        let t: Cp = data.read(i + offset).powf(2.0);
        data.write(t, i + offset);
    }

    // reverse fft, dir = -1
    fft_parallel(config, &mut data, -1);
    comm::barrier();

    /* debug */ if DBG { print_array(config, &data, n, "output"); }
}

fn fft_parallel(config: &mut Config, data: &mut Array<Cp>, dir: i8) {
    let rankn: usize = config.rankn as usize;
    let rank: usize = config.rank as usize;

    let mut w: Cp = Complex::from_polar(&(1.0), &(-dir as f64 * PI));

    let mut stride: usize = 1;
    let n: usize = data.local_size;
    let N: usize = n * rankn;
    let offset: usize = n * rank;

    bit_reverse(config, data, n);
    comm::barrier();
//    /* debug */ { print_array(&data, rank, n); }

    while stride < N {
        if stride < n { step_sequential(config, data, &mut w, stride); }
        else { step_parallel(config, data, &mut w, stride); }
        stride <<= 1;
    }
    comm::barrier();

    if dir == -1 {
        for i in 0 .. n {
            let mut t: Cp = data.read(offset + i) / (N as f64);
            data.write(t, offset + i);
        }
    }
    comm::barrier();
}

fn bit_reverse(config: &mut Config, data: &mut Array<Cp>, size: usize) {
    let rankn: usize = config.rankn as usize;
    let rank: usize = config.rank as usize;
    let n = data.local_size;
    let offset = rank * n;
    let logN = ((rankn * n) as f64).log2() as usize;

    let mut idx = 0;
    let mut buf: Array<Cp> = Array::init(config, rankn);

    for i in 0 .. n {
        idx = i + offset;
        let idx_rev = rev(idx, logN);

        if idx < idx_rev {
            let t = data.read(idx);
            let r = data.read(idx_rev);
            data.write(r, idx);
            data.write(t, idx_rev);
        }
    }
}

fn rev(idx: usize, logN: usize) -> usize {
    let mut idx_rev = 0;
    for j in 0 .. logN {
        if (idx & (1 << (logN - j - 1))) != 0 {
            idx_rev |= (1 << j);
        }
    }
    idx_rev
}

fn step_parallel(config: &mut Config, data: &mut Array<Cp>, w: &mut Cp, stride: usize) {
    let rankn: usize = config.rankn as usize;
    let rank: usize = config.rank as usize;
    let mut wk: Cp = Complex::new(1.0, 0.0);
    let n = data.local_size;
    let offset = rank * n;

    wk *= *(&w.powf((offset % stride) as f64));

    let mut idx = 0;
    for i in 0 .. n {
        idx = i + offset;

        if (idx & stride) == 0 {
            // if 0 on stride bit (on the left)
            let t: Cp = data.read(idx);
            let r: Cp = wk * data.read(idx + stride);
            data.write(t + r, idx);
            data.write(t - r, idx + stride);
        }

        wk *= *w;
    }

    *w = *(&w.sqrt());
    comm::barrier();
}

fn step_sequential(config: &mut Config, data: &mut Array<Cp>, w: &mut Cp, stride: usize) {
    let rankn: usize = config.rankn as usize;
    let rank: usize = config.rank as usize;
    let mut wk: Cp;
    let n = data.local_size;
    let offset = rank * n;

    let mut p = 0;
    while p < n {
        wk = Complex::new(1.0, 0.0);
        for k in 0 .. stride {
            let idx = offset + p + k;

            let mut t: Cp = data.read(idx);
            let mut r: Cp = wk * data.read(idx + stride);
            // if 0 on (stride >> 1) bit (on the left)
            data.write(t + r, idx);
            // if 1 on (stride >> 1) bit (on the right)
            data.write(t - r, idx + stride);

            wk *= *w;
        }
        p += 2 * stride;
    }

    *w = *(&w.sqrt());
    comm::barrier();
}

fn print_array(config: &mut Config, data: &Array<Cp>, n: usize, msg: &str) {
    let rank = config.rank;
    let offset = rank * n;

    let data_serial = data.get_ptr(offset).arget(n);
    let data_real: Vec<i32> = data_serial.iter().map(|x| x.re.round() as i32).collect();
    println!("{}: rank {}: = {:?}\n", msg, rank, data_real);
    comm::barrier();
}

fn print_array_all(config: &mut Config, data: &Array<Cp>, n: usize, msg: &str) {
    let rank = config.rank;
    let rankn = config.rankn;
    let offset = rank * n;
    let N = rankn * n;

    let mut data_ptrs: Vec<GlobalPointer<Cp>> = Vec::new();
    data_ptrs.resize(rankn, GlobalPointer::null());
    data_ptrs[rank] = data.get_ptr(offset);
    comm::barrier();
    let mut output_ptr: GlobalPointer<Cp> = GlobalPointer::init(config, N);
    comm::gather(&mut output_ptr, &mut data_ptrs[rank], 0, n);
    comm::barrier();

    if rank == 0 {
        let data_serial = output_ptr.arget(N);
        let data_real: Vec<i32> = data_serial.iter().map(|x| x.re.round() as i32).collect();
        println!("{}: {:?}\n", msg, data_real);
    }
    comm::barrier();
}