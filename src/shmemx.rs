#![allow(dead_code)]
use shmemx;
pub extern crate libc;
use self::libc::{c_int, size_t, c_long};
use std::slice;

#[link(name="oshmem", kind="dylib")]
extern {
    fn shmem_init();
    fn shmem_finalize();
    fn shmem_n_pes() -> c_int;
    fn shmem_my_pe() -> c_int;
    fn shmem_barrier_all();
    pub fn shmem_malloc(size: size_t) -> *mut u8;
    pub fn shmem_free(ptr: *mut u8);
    pub fn shmem_putmem(target: *mut u8, source: *const u8, len: size_t, pe: c_int);
    pub fn shmem_getmem(target: *mut u8, source: *const u8, len: size_t, pe: c_int);
    pub fn shmem_broadcast64(target: *mut u64, source: *const u64, nelems: size_t, PE_root: c_int,
                             PE_start: c_int, logPE_stride: c_int, PE_size: c_int, pSync: *mut c_long); // how to denote *long?
    // added by lfz
    pub fn shmem_int_cswap(target: *mut c_int, cond: c_int, value: c_int, pe: c_int) -> c_int;
    pub fn shmem_long_cswap(target: *mut c_long, cond: c_long, value: c_long, pe: c_int) -> c_long;
    pub fn shmem_int_finc(target: *mut c_int, pe: c_int) -> c_int;
}

pub static _SHMEM_SYNC_VALUE: c_long = -1;
pub static _SHMEM_BCAST_SYNC_SIZE: usize = 2;
pub fn init() {
    unsafe {
        shmem_init();
    }
}

pub fn finalize() {
    unsafe {
        shmem_finalize();
    }
}

pub fn n_pes() -> usize {
    unsafe {
        let npes = shmem_n_pes() as usize;
        npes
    }
}

pub fn my_pe() -> usize {
    unsafe {
        let mype = shmem_my_pe() as usize;
        mype
    }
}

pub fn barrier() {
    unsafe {
        shmem_barrier_all();
    }
}


fn test_shmem() {
    // The statements here will be executed when the compiled binary is called
    // Print text to the console
    shmemx::init();
    let my_pe = shmemx::my_pe();
    let n_pes = shmemx::n_pes();

    unsafe {
        let source_ptr: *mut u8 = shmem_malloc(n_pes);
        let target_ptr = shmem_malloc(n_pes);
        let source_slice = slice::from_raw_parts_mut(source_ptr, n_pes);
        let target_slice = slice::from_raw_parts_mut(target_ptr, n_pes);

        for i in 0..n_pes {
            source_slice[i] = i as u8;
        }

        for i in 0..n_pes {
            target_slice[i] = 0 as u8;
        }

        if my_pe == 0 {
            shmem_putmem(target_ptr, source_ptr, n_pes / 2, 1);
        }
        if my_pe == 1 {
            shmem_getmem(target_ptr.add(n_pes / 2), source_ptr.add(n_pes / 2), n_pes / 2, 0);
        }
        shmemx::barrier();

        if my_pe == 1 {
            println!("Hello World! I am process {} out of {}",
             shmemx::my_pe(), shmemx::n_pes());
            for i in 0..n_pes {
                print!(" {}", target_slice[i]);
            }
            println!();
        }

        shmem_free(source_ptr);
        shmem_free(target_ptr);
    }
    shmemx::barrier();
    shmemx::finalize();
}