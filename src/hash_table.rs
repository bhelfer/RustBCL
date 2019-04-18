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

#[derive(Debug, Copy, Clone)]
pub struct HashEntry<K, V> {
    key: K,
    value: V,
    used: bool,
}

impl<K, V> HashEntry<K, V>
    where K: Clone + Hash + Hasher + Copy,
          V: Clone + Copy,
{
    fn new(key: K, value: V) -> Self {
        Self { key, value, used: false, }
    }
    fn set(&mut self, key: K, value: V) {
        self.key = key;
        self.value = value;
    }
}

type HE<K, V> = HashEntry<K, V>;

#[derive(Debug, Clone)]
pub struct HashTable<K, V> {
    pub local_size: usize,
    pub hash_table: Vec<GlobalPointer<HE<K, V>>>,
}

impl<K, V> HashTable<K, V>
    where K: Clone + Hash + Hasher + Copy,
          V: Clone + Copy,
{
    pub fn new(config: &mut Config, n: usize) -> Self {
        let local_size = (n + shmemx::n_pes() - 1) / config.rankn;

        let mut hash_table: Vec<GlobalPointer<HE<K, V>>> = Vec::new();
        hash_table.resize(shmemx::n_pes(), GlobalPointer::null());

        config.barrier();

        hash_table[config.rank] = config.alloc::<HE<K, V>>(local_size);

        config.barrier();

        for rank in 0..config.rankn {
            comm::broadcast(&mut hash_table[rank], rank);
        }

        config.barrier();

        Self {
            local_size,
            hash_table,
        }
    }

    fn get_entry(&self, slot: usize) -> HE<K, V> {
        let node = slot / self.local_size;
        let node_slot = slot - node * self.local_size;

        (self.hash_table[node] + node_slot).rget()
    }

    fn set_entry(&self, entry: HE<K, V>, slot: usize) {
        let node = slot / self.local_size;
        let node_slot = slot - node * self.local_size;

        (self.hash_table[node] + node_slot).rput(entry);
    }

    pub fn insert(self, mut key: K, value: V) {
        let mut hasher =  DefaultHasher::new();
        Hash::hash(&key, &mut hasher);
        let hash = hasher.finish();
        let mut probe: u64 = 0;
        let mut success = false;
        loop {
            let slot: usize = ((hash + probe) % (self.local_size as u64)) as usize;
            probe += 1;
            if success {
                let mut entry: HE<K, V> = self.get_entry(slot);
                entry.set(key, value);
                self.set_entry(entry, slot);
            }
        }
    }
}