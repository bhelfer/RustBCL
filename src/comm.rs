use std::mem::size_of;

use shmemx;
use shmemx::libc::{c_long, c_void};

//noinspection RsAssignToImmutable
pub fn broadcast<T>(val: &mut T, root: usize) {
    unsafe{
        let p_sync_ptr: *mut c_long= shmemx::shmem_malloc(shmemx::_SHMEM_BCAST_SYNC_SIZE * size_of::<c_long>()) as *mut c_long;
        if p_sync_ptr.is_null() {
            panic!("BCL: Could not allocate shared memory segment.")
        }
        let p_sync_slice = std::slice::from_raw_parts_mut(p_sync_ptr, shmemx::_SHMEM_BCAST_SYNC_SIZE);
        for i in 0..shmemx::_SHMEM_BCAST_SYNC_SIZE {
            p_sync_slice[i] = shmemx::_SHMEM_SYNC_VALUE;
        }
//        shmemx::barrier();

        let nelem = (size_of::<T>() + 7) / 8;
        println!("broadcast: size<t>:{}, nelem={}", size_of::<T>(), nelem);
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
        shmemx::barrier();
        shmemx::shmem_broadcast64(rv, startval, nelem, root as i32, 0, 0, shmemx::n_pes() as i32, p_sync_ptr);
        shmemx::barrier();
        if shmemx::my_pe() != root {
            libc::memcpy(val as *mut T as *mut c_void, rv as *const c_void, size_of::<T>());
        }
        shmemx::barrier();
        shmemx::shmem_free(p_sync_ptr as *mut u8);
        shmemx::shmem_free(rv as *mut u8);
        shmemx::shmem_free(startval as *mut u8);
    }
}