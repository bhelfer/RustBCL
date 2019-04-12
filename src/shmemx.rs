#![allow(dead_code)]

extern crate libc;
use self::libc::{c_int, size_t};

#[link(name="oshmem", kind="dylib")]
extern {
    fn shmem_init();
    fn shmem_finalize();
    fn shmem_n_pes() -> libc::c_int;
    fn shmem_my_pe() -> libc::c_int;
    fn shmem_barrier_all();
    fn shmem_malloc(size: libc::size_t) -> *mut u8;
    fn shmem_free(ptr: *mut u8);
    fn shmem_putmem(target: *mut u8, source: *const u8, len: size_t, pe: c_int);
    fn shmem_getmem(target: *mut u8, source: *const u8, len: size_t, pe: c_int);
}

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
        let npes: usize = shmem_n_pes() as usize;
        npes
    }
}

pub fn my_pe() -> usize {
    unsafe {
        let mype: usize = shmem_my_pe() as usize;
        mype
    }
}

pub fn barrier() {
    unsafe {
        shmem_barrier_all();
    }
}
