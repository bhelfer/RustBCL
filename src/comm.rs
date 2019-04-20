use shmemx;
use std::mem::size_of;
use shmemx::libc::{c_long, c_void, c_int};
use global_pointer::GlobalPointer;

pub fn broadcast<T>(val: &mut T, root: usize) {
    unsafe{
        let p_sync_ptr: *mut c_long = shmemx::shmem_malloc(shmemx::_SHMEM_BCAST_SYNC_SIZE * size_of::<c_long>()) as *mut c_long;
        if p_sync_ptr.is_null() {
            panic!("BCL: Could not allocate shared memory segment.")
        }
        let p_sync_slice = std::slice::from_raw_parts_mut(p_sync_ptr, shmemx::_SHMEM_BCAST_SYNC_SIZE);
        for i in 0..shmemx::_SHMEM_BCAST_SYNC_SIZE {
            p_sync_slice[i] = shmemx::_SHMEM_SYNC_VALUE;
        }
//        shmemx::barrier();

        let nelem = (size_of::<T>() + 7) / 8;
//        println!("broadcast: size<t>:{}, nelem={}", size_of::<T>(), nelem);
        let rv = shmemx::shmem_malloc(nelem * 64) as *mut u64;
        if rv.is_null() {
            panic!("BCL: Could not allocate shared memory segment.")
        }
        let startval = shmemx::shmem_malloc(nelem * 64) as *mut u64;
        if startval.is_null() {
            panic!("BCL: Could not allocate shared memory segment.")
        }

        if shmemx::my_pe() == root {
            libc::memcpy(startval as *mut c_void, val as *const T as *const c_void, size_of::<T>());
        }
        shmemx::shmem_broadcast64(rv, startval, nelem, root as i32, 0, 0, shmemx::n_pes() as i32, p_sync_ptr);
        if shmemx::my_pe() != root {
            libc::memcpy(val as *mut T as *mut c_void, rv as *const c_void, size_of::<T>());
        }
        shmemx::shmem_free(p_sync_ptr as *mut u8);
        shmemx::shmem_free(rv as *mut u8);
        shmemx::shmem_free(startval as *mut u8);
    }
}

// added by lfz
pub fn int_compare_and_swap(ptr: &mut GlobalPointer<c_int>, old_val: c_int, new_val: c_int) -> c_int {

    let rank = ptr.rank;
    unsafe {
        shmemx::shmem_int_cswap(
            ptr.rptr() as *mut c_int,
            old_val as c_int,
            new_val as c_int,
            rank as c_int
        )
    }
}

// added by lfz
pub fn long_compare_and_swap(ptr: &mut GlobalPointer<c_long>, old_val: c_long, new_val: c_long) -> c_long {

    let rank = ptr.rank;
    unsafe {
        shmemx::shmem_long_cswap(
            ptr.rptr() as *mut c_long,
            old_val as c_long,
            new_val as c_long,
            rank as c_int
        )
    }
}

// added by lfz
pub fn int_finc(ptr: &mut GlobalPointer<i32>) -> i32 {
    let rank = ptr.rank;
    unsafe {
        shmemx::shmem_int_finc(
            ptr.rptr() as *mut i32,
            rank as c_int
        )
    }
}