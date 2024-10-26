use std::borrow::Borrow;
use std::collections::{hash_map::DefaultHasher, LinkedList};
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

// The ConcurrentMultiMap struct is a concurrent hash map that allows multiple values to be
// associated with a single key. It is implemented using a vector of RwLocks, where each lock
// protects a linked list of key-value pairs.
pub struct ConcurrentMultiMap<K: Hash + Eq, V> {
    buckets: Vec<RwLock<LinkedList<(K, V)>>>,
}

impl<K: Hash + Eq, V> ConcurrentMultiMap<K, V> {
    // TODO:
    // Create a new empty ConcurrentMultiMap with the given number of buckets.
    pub fn new(bucket_count: usize) -> Self {
        todo!()
    }
}

impl<K: Hash + Eq, V: Clone + Eq> ConcurrentMultiMap<K, V> {
    // TODO:
    // Associate the given value with the given key. To do so, hash the key, and find the
    // corresponding bucket in the vector by modulo-ing the hash by the number of buckets. Then,
    // take a writer lock of the bucket and iterate over the linked list, checking if the
    // key-values pair already exists. If it does, return early. Otherwise, add the key-value pair
    // to the linked list.
    pub fn set(&self, key: K, value: V) {
        todo!()
    }

    // TODO:
    // Retrieve all values associated with `key`. To do so, hash the key, and find the
    // corresponding bucket in the vector by modulo-ing the hash by the number of buckets. Then,
    // take a reader lock of the bucker and iterate over the linked list, collecting all values
    // associated with the key by `clone`-ing them. Return the collected values.
    pub fn get<Q>(&self, key: &Q) -> Vec<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        todo!()
    }
}

