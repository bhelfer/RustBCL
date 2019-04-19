#![allow(dead_code)]
#![allow(unused)]

use global_pointer;
use comm;
use config;
use config::Config;
use shmemx;
use std::marker::PhantomData;
use global_pointer::GlobalPointer;
use shmemx::shmem_broadcast64;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::ptr::null;

#[derive(Debug, Copy, Clone)]
struct HashEntry<K, V> {
    key: K,
    value: V,
}

impl<K, V> HashEntry<K, V>
    where K: Clone + Hash + Copy + Default + Debug + PartialEq<K>,
          V: Clone + Copy + Default + PartialEq<V>,
{
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
    pub fn set(&mut self, key: K, value: V) {
        self.key = key;
        self.value = value;
    }
    pub fn get_key(&self) -> K {
        self.key
    }
    pub fn get_value(&self) -> V {
        self.value
    }
}

type HT<K, V> = HashTable<K, V>;
type HE<K, V> = HashEntry<K, V>;

#[derive(Debug, Clone)]
pub struct HashTable<K, V> {
    pub size: usize,
    pub local_size: usize,
    hash_table: Vec<GlobalPointer<HE<K, V>>>,
    used: Vec<GlobalPointer<u32>>,
    free_flag: u32,
    reserved_flag: u32,
    ready_flag: u32,
}

impl<K, V> HashTable<K, V>
    where K: Clone + Hash + Copy + Default + Debug + PartialEq<K>,
          V: Clone + Copy + Default + Eq + PartialEq<V>,
{
    pub fn new(config: &mut Config, size: usize) -> Self {
        let local_size = (size + shmemx::n_pes() - 1) / config.rankn;

        let free_flag: u32 = 0;
        let reserved_flag: u32 = 1;
        let ready_flag: u32 = 2;

        // used record GlobalPointer
        let mut used: Vec<GlobalPointer<u32>> = Vec::new();
        used.resize(shmemx::n_pes(), GlobalPointer::null());
        used[config.rank] = config.alloc::<u32>(local_size);
        config.barrier();

        let local_used: *mut u32 = used[config.rank].local();

        for i in 0..local_size {
            unsafe { local_used.add(i).write(free_flag); }
        }
        config.barrier();

        // hash entry GlobalPointer
        let mut hash_table: Vec<GlobalPointer<HE<K, V>>> = Vec::new();
        hash_table.resize(shmemx::n_pes(), GlobalPointer::null());
        hash_table[config.rank] = config.alloc::<HE<K, V>>(local_size);
        config.barrier();

        hash_table[config.rank] = config.alloc::<HE<K, V>>(local_size);
        config.barrier();

        let local_table: *mut HE<K, V> = hash_table[config.rank].local();

        for i in 0..local_size {
            unsafe {
                local_table.add(i).write(HashEntry::new(
                    Default::default(), Default::default()
                ));
            }
        }
        config.barrier();

        for rank in 0..config.rankn {
            comm::broadcast(&mut hash_table[rank], rank);
        }
        config.barrier();

        Self {
            size,
            local_size,
            hash_table,
            used,
            free_flag,
            reserved_flag,
            ready_flag,
        }
    }

    fn get_entry(&self, slot: usize) -> HE<K, V> {
        let node = slot / self.local_size;
        let node_slot = slot - node * self.local_size;

        if node >= shmemx::n_pes() { panic!("HashTable::get_entry: node {} out of bound!", node); }
        if node_slot >= self.local_size { panic!("HashTable::get_entry: node_slot {} out of bound!", node_slot); }

        (self.hash_table[node] + node_slot).rget()
    }

    fn set_entry(&self, slot: usize, entry: HE<K, V>) {
        let node = slot / self.local_size;
        let node_slot = slot - node * self.local_size;

        if node >= shmemx::n_pes() { panic!("HashTable::get_entry: node {} out of bound!", node); }
        if node_slot >= self.local_size { panic!("HashTable::get_entry: node_slot {} out of bound!", node_slot); }

        (self.hash_table[node] + node_slot).rput(entry);
    }

    /* Request slot for key. If slot's free, take it.
       If slot's taken (ready_flag), reserve it (reserve_flag),
       so that you can write to it. */
    fn request_slot(&self, slot: usize, key: K) -> bool {
        let node = slot / self.local_size;
        let node_slot = slot - node * self.local_size;
        let used_ptr = self.used[node] + node_slot;
        println!("rank, node, node_slot = {}, {}, {}", shmemx::my_pe(), node, node_slot);

        let mut flag_used = self.free_flag;
        loop {
            flag_used = comm::int_compare_and_swap(used_ptr, self.ready_flag, self.reserved_flag);
            if flag_used != self.reserved_flag { break; }
        }

        if !flag_used == self.free_flag || flag_used == self.ready_flag {
            panic!("HashMap rqs: used flag was somehow corrupted (-> reserved_flag).");
        }

        if flag_used == self.ready_flag {
            if self.get_entry(slot).get_key() == key {
                loop {
                    flag_used = comm::int_compare_and_swap(used_ptr, self.ready_flag, self.reserved_flag);
                    if flag_used == self.ready_flag { break; }
                }
                return true;
            } else {
                return false;
            }
        } else {
            return true;
        }
    }

    pub fn insert(self, mut key: K, value: V) -> bool {
        let mut hasher =  DefaultHasher::new();
        Hash::hash(&key, &mut hasher);
        let hash = hasher.finish();
        let mut probe: u64 = 0;
        let mut success = false;

        loop {
            let slot: usize = ((hash + probe) % (self.size as u64)) as usize;
            probe += 1;

            println!("Requesting slot {}, key = {:?}", slot, key);
            success = self.request_slot(slot, key);

            if success {
                let mut entry: HE<K, V> = self.get_entry(slot);
                entry.set(key, value);
                self.set_entry(slot, entry);
            }

            if success || probe >= self.size as u64 { break; }
        }

        success
    }
}