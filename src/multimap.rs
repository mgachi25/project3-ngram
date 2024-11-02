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
        let buckets = (0..bucket_count)
            .map(|_| RwLock::new(LinkedList::new()))
            .collect();
        
        ConcurrentMultiMap { buckets }
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
        // Hash the key to find the bucket index
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let bucket_index = (hash % self.buckets.len() as u64) as usize;
    
        // Acquire write lock for the bucket
        let mut bucket = self.buckets[bucket_index].write().unwrap();
    
        // Check if the key-value pair already exists
        for (existing_key, existing_value) in bucket.iter() {
            if *existing_key == key && *existing_value == value {              
                return;
            }
        }
    
        // If not, insert the new key-value pair
        bucket.push_back((key, value));
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
        // Hash the key to find the bucket index
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let bucket_index = (hash % self.buckets.len() as u64) as usize;

        // Acquire read lock for the bucket
        let bucket = self.buckets[bucket_index].read().unwrap();

        // Collect all values associated with the key by cloning them
        bucket
            .iter()
            .filter_map(|(existing_key, value)| {
                if existing_key.borrow() == key {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

