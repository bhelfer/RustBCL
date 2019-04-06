extern crate libc;

#[link(name="sma", kind="static")]
#[link(name="pmi_simple", kind="static")]
extern {
    fn shmem_init();
    fn shmem_finalize();
    fn shmem_n_pes() -> libc::c_int;
    fn shmem_my_pe() -> libc::c_int;
    fn shmem_barrier_all();
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
