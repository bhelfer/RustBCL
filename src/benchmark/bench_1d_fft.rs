//#![allow(dead_code)]
//#![allow(unused)]
//#![allow(deprecated)]
//#![allow(non_snake_case)]
//
//use num::complex::{Complex, Complex32, Complex64};
//use rand::{rngs::StdRng, Rng, thread_rng, SeedableRng, ChaChaRng};
//
//use base::{global_pointer::GlobalPointer, config::Config};
//use backend::comm;
//
//use containers::array;
//
//use std::time::{SystemTime, Duration};
//use std::vec::Vec;
//use std::env;
//use std::f32::consts::PI;
//use containers::array::Array;
//
//type Cp = Complex32;
//
//pub fn benchmark_1d_fft(config: &mut Config) {
//
//    let args: Vec<String> = env::args().collect();
//    let mut n: usize;
//    let min_size: usize = (1 << 2);
//    // output debug info or not
//    let mut DBG: bool = true;
//
//    let rankn: usize = config.rankn as usize;
//    let rank: usize = config.rank as usize;
//
//    if args.len() <= 1 {
//        n = min_size;
//        println!("not enough arguments\nUse default argument n = {}", n);
//    } else {
//        n = args[1].clone().parse().unwrap();
//        n /= rankn;
//        n = (n as f32).log2().ceil() as usize;
//        if n < min_size { n = min_size; }
//    }
//
//    if n >= 100 { DBG = false; }
//
//    let rankn: usize = config.rankn as usize;
//    let rank: usize = config.rank as usize;
//    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);
//
//    let size: usize = n * rankn;
//
//    let mut data = Array::init(config, size);
//    for i in 0 .. data.local_size {
//        data.write(Complex::new(1.0, 0.0), i);
//    }
//    comm::barrier();
//
//    /* debug */ if DBG {
//        let data_serial = data.get_ptr(0).arget(data.local_size);
//        println!("rank {}: data = {:?}", rank, data_serial); comm::barrier();
//    }
//
//    // here start the parallel 1d fft
//    fft_parallel(config, &mut data, size);
//
//
//}
//
//fn fft_parallel(config: &mut Config, data: &mut Array<Cp>, size: usize) {
//    let rankn: usize = config.rankn as usize;
//
//    let mut w: Cp = Complex::from_polar(&(1.0), &(-PI));
//    bit_reverse_parallel(config, data, size);
//
//    let mut stride: usize = 2;
//    let N: usize = (*size as f32).log2().ceil() as usize;
//    let n: usize = N / rankn;
//
//    while stride < N {
//        if stride < n { step_sequential(config, data, stride, &mut w); }
//        else { step_parallel(config, data, stride, &mut w); }
//        stride *= 2;
//    }
//}
//
//fn bit_reverse_parallel(config: &mut Config, data: &mut Array<Cp>, size: usize) {
//    let rankn: usize = config.rankn as usize;
//    let rank: usize = config.rank as usize;
//
//}
//
//fn step_parallel(config: &mut Config, data: &mut Array<Cp>, stride: usize, w: &mut Cp) {
//    let rankn: usize = config.rankn as usize;
//    let rank: usize = config.rank as usize;
//    let mut wk: Cp;
//    let n = data.local_size;
//    let offset = rank * n;
//
//    for i in 0 .. ((rank * data) % stride) {
//        wk *= w;
//    }
//
//    let mut global_i = 0;
//    for i in 0 .. n {
//        global_i = i + offset;
//
//        let t: Cp;
//        if (global_i & stride) == 0 {
//            t = data[i];
//
//        }
//
//    }
//
//    *w = *(&w.sqrt());
//}
//
///*
//void  stepPar (vector <complex < double >> & data, complex < double > & w, int d) {
//    int indGlobal;
//    complex < double > oldElement;
//    for ( int k = 0 ; k <nloc; k ++) {
//        indGlobal = k + myPE * nloc;    // Overall index in the even part
//
//        if ((indGlobal & ( 0x1 << ( int ) log2 (d))) == 0 ) {
//            oldElement = data [k];
//            swapPar (data [k], myPE ^ (d / nloc));
//            data [k] = data [k] + oldElement;
//        }
//        else {
//            data [k] = wk * data [k];
//            oldElement = data [k];
//            swapPar (data [k], myPE ^ (d / nloc));
//            data [k] = data [k] - oldElement;
//        }
//        wk * = w;
//    }
//    w = sqrt (w);
//}
//*/
//
//fn step_sequential(config: &mut Config, data: &mut Array<Cp>, stride: usize, w: &mut Cp) {
//    let rankn: usize = config.rankn as usize;
//    let rank: usize = config.rank as usize;
//    let mut wk: Cp;
//    let n = data.local_size;
//    let m = stride / 2;
//
//    let mut p = 0;
//    while p < n {
//        wk = Complex::new(1.0, 0.0);
//        for k in 0 .. m {
//            let t: Cp = wk * data[k + m];
//            data[p + k + m] = data[p + k] - t;
//            data[p + k] = data[p + k] + t;
//            wk *= w;
//        }
//        p += stride;
//    }
//
//    *w = *(&w.sqrt());
//}
//
//fn swap_parallel(config: &mut Config, &mut loc: Cp, proc: usize) {
//    let rank: usize = config.rank as usize;
//    let mut buf: Array<Cp> = Array::init(config, 1);
//
//    buf.write(loc, proc);
//    comm::barrier();
//
//    *loc = buf.read(rank);
//    comm::barrier();
//}
//
////void  swapPar (complex < double > & loc, int proc) {
////    // Get process id
////    int myPE;
////    MPI_Comm_rank (MPI_COMM_WORLD, & myPE);
////
////    // Init of buffer to send and receive the complex number to send
////    double * send_buf = new  double [ 2 ];
////    double * recv_buf = new  double [ 2 ];
////
////    // Put in the buffer the complex number
////    send_buf [ 0 ] = loc. real ();
////    send_buf [ 1 ] = loc. imag ();
////
////    // Determine an order for send and receive
////    if (proc> myPE) {
////        MPI_Send (send_buf, 2 , MPI_DOUBLE, proc, 0 , MPI_COMM_WORLD);
////        MPI_Recv (recv_buf, 2 , MPI_DOUBLE, proc, 0 , MPI_COMM_WORLD, MPI_STATUS_IGNORE);
////    } else {
////        MPI_Recv (recv_buf, 2 , MPI_DOUBLE, proc, 0 , MPI_COMM_WORLD, MPI_STATUS_IGNORE);
////        MPI_Send (send_buf, 2 , MPI_DOUBLE, proc, 0 , MPI_COMM_WORLD);
////    }
////
////    // Update the loc data
////    complex < double > tmpLoc (recv_buf [ 0 ], recv_buf [ 1 ]);
////    loc = tmpLoc;
////}