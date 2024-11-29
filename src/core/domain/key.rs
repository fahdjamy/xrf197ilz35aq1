use rand::distr::Alphanumeric;
use rand::{rng, Rng};

pub fn generate_unique_key(length: usize) -> String {
    let unique_key = rng()
        .sample_iter(&Alphanumeric) // generates random chars from the Alphanumeric distribution
        .take(length) // limits the length of the generated string to 'length' chars
        .map(char::from) // convert the generated random numbers to chars
        .collect(); // collect the generated chars into a String

    unique_key
}

#[cfg(test)]
mod tests {
    use crate::core::domain::key::generate_unique_key;
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_generate_unique_key() {
        let unique_key = generate_unique_key(10);
        assert_eq!(unique_key.len(), 10);
    }

    #[test]
    fn test_generate_unique_key_alphanumeric() {
        let key = generate_unique_key(32);
        assert!(key.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_unique_key_concurrent() {
        let key_length = 32;
        let num_threads = 10; // number of threads to simulate concurrent execution
        let num_keys_per_thread = 1000; // Each thread will generate this many keys.
        let keys = Arc::new(Mutex::new(HashSet::new()));

        let mut handles = vec![];

        for _ in 0..num_threads {
            // clone the Arc - keys to avoid problem that arise when threads try to move this single Arc into their closures
            // cloning creates a new Arc that points to the same Mutex-protected HashSet
            // Arc allows multiple owners for the same data by keeping track of how many references (clones) exist to the data
            let keys = Arc::clone(&keys);
            let handle = thread::spawn(move || {
                // local HashSet is created for each thread to track the keys it generates.
                // This allows for a quick preliminary check for duplicates within the thread itself
                let mut local_keys = HashSet::new();
                for _ in 0..num_keys_per_thread {
                    let key = generate_unique_key(key_length);
                    if !local_keys.insert(key.clone()) {
                        panic!("Duplicate key generated within a thread");
                    }
                    // Acquires the lock on the shared HashSet. This is critical for thread safety
                    let mut global_keys = keys.lock().unwrap();
                    if !global_keys.insert(key) {
                        panic!("Duplicate key generated across threads");
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let keys = keys.lock().unwrap();
        assert_eq!(keys.len(), num_threads * num_keys_per_thread);
    }
}
