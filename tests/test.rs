use quickcheck::quickcheck;
const THREADS: usize = 16;

// ============================ MULTIMAP ============================
mod test_multimap {
    use super::*;
    use ngram::multimap::*;
    #[derive(PartialEq, Eq, Hash)]
    struct UnCloneable(i32);
    #[test]
    fn test_new_is_empty_5() {
        fn new_is_empty(n: i32) {
            let map = ConcurrentMultiMap::<UnCloneable, usize>::new(10);
            assert_eq!(map.get(&UnCloneable(n)).len(), 0);
        }
        quickcheck(new_is_empty as fn(i32));
    }
    #[test]
    fn test_get_after_set_single_5() {
        fn get_after_set_single(k: i32, v: usize) {
            let map = ConcurrentMultiMap::<UnCloneable, usize>::new(10);
            map.set(UnCloneable(k), v as usize);
            assert_eq!(map.get(&UnCloneable(k)), vec![v as usize]);
        }
        quickcheck(get_after_set_single as fn(i32, usize));
    }
    #[test]
    fn test_get_after_set_multi_5() {
        use std::collections::HashSet;
        fn get_after_set_multi(k: i32, values: HashSet<usize>) {
            let map = ConcurrentMultiMap::<UnCloneable, usize>::new(10);
            for values in values.iter() {
                map.set(UnCloneable(k), *values as usize);
            }
            let result = map.get(&UnCloneable(k));
            println!("+==================+");
            println!("{:?}", k);
            println!("{:?}", values);
            println!("{:?}", result);
            assert_eq!(result.len(), values.len());
            for values in values.iter() {
                assert!(result.contains(&(*values as usize)));
            }
        }
        quickcheck(get_after_set_multi as fn(i32, HashSet<usize>));
    }
    #[test]
    fn test_get_from_large_map_5() {
        fn get_from_large_map(k: i32, v: usize, others: Vec<(i32, usize)>) {
            let map = ConcurrentMultiMap::<UnCloneable, usize>::new(1000);
            for (k, v) in others.iter() {
                map.set(UnCloneable(*k), *v as usize);
            }
            map.set(UnCloneable(k), v as usize);
            assert!(map.get(&UnCloneable(k)).contains(&(v as usize)));
        }
        quickcheck(get_from_large_map as fn(i32, usize, Vec<(i32, usize)>));
    }
    #[test]
    fn test_no_duplicates_5() {
        fn no_duplicates(k: i32, v: usize) {
            let map = ConcurrentMultiMap::<UnCloneable, usize>::new(10);
            map.set(UnCloneable(k), v as usize);
            map.set(UnCloneable(k), v as usize);
            map.set(UnCloneable(k), v as usize);
            map.set(UnCloneable(k), v as usize);
            assert_eq!(map.get(&UnCloneable(k)), vec![v as usize]);
        }
        quickcheck(no_duplicates as fn(i32, usize));
    }
    #[test]
    fn passes_stress_test_10() {
        fn passes_stress_test(tuples: Vec<(i32, usize, bool)>) {
            use std::sync::Arc;
            let thread_count = 20;
            let chunk_size = tuples.len() / thread_count;
            if chunk_size == 0 {
                return;
            }
            let tuples_chunked = tuples.chunks(chunk_size).map(Vec::from).collect::<Vec<_>>();

            let map = Arc::new(ConcurrentMultiMap::<UnCloneable, usize>::new(128));
            let threads = tuples_chunked.into_iter().map(|chunk| {
                let map = Arc::clone(&map);
                std::thread::spawn(move || {
                    for (k, v, is_write) in chunk.iter() {
                        if *is_write {
                            map.set(UnCloneable(*k), *v as usize);
                        } else {
                            map.get(&UnCloneable(*k));
                        }
                    }
                })
            });
            threads.into_iter().for_each(|t| t.join().unwrap());
        }
        quickcheck(passes_stress_test as fn(Vec<(i32, usize, bool)>));
    }
}

// ============================ POOL ============================
mod test_pool {
    use ngram::pool::*;
    use std::sync::{Arc, Mutex};
    #[test]
    fn test_uses_multiple_threads_5() {
        let pool = ThreadPool::new(4);

        // purposefully deadlock one of the threads in the thread pool
        pool.execute(move || loop {});

        // Make sure there is some other thread that is still able to run
        // and send a message back to this thread
        let (tx, rx) = std::sync::mpsc::channel();
        pool.execute(move || {
            tx.send(()).unwrap();
        });
        match rx.recv() {
            Ok(_) => {}
            Err(_) => assert!(false, "thread did not make progress"),
        }

        // avoid calling drop on the pool so we don't wait for the deadlocked thread
        std::mem::forget(pool);
    }

    #[test]
    fn test_joins_successfully_5() {
        let pool = ThreadPool::new(4);
        let counter = Arc::new(Mutex::new(0));
        for _ in 0..8 {
            let counter = Arc::clone(&counter);
            pool.execute(move || {
                let mut counter = counter.lock().unwrap();
                *counter += 1;
            });
        }

        // wait for all threads to finish
        drop(pool);
        assert_eq!(*counter.lock().unwrap(), 8);
    }
}

// ============================ SERIALIZE ============================
mod test_serialize {
    use super::*;
    use ngram::message::*;
    #[test]
    fn test_round_trip_request_5() {
        fn round_trip_request(s: String, n: usize) {
            let pub_request = Request::Publish { doc: s.clone() };
            let search_request = Request::Search { word: s };
            let retrieve_request = Request::Retrieve { id: n };
            assert_eq!(
                Request::from_bytes(&pub_request.to_bytes()[..]).unwrap(),
                pub_request
            );
            assert_eq!(
                Request::from_bytes(&search_request.to_bytes()[..]).unwrap(),
                search_request
            );
            assert_eq!(
                Request::from_bytes(&retrieve_request.to_bytes()[..]).unwrap(),
                retrieve_request
            );
        }
        quickcheck(round_trip_request as fn(String, usize));
    }

    #[test]
    fn test_round_trip_response_5() {
        fn round_trip_response(s: String, n: usize) {
            let pub_response = Response::PublishSuccess(n);
            let search_response = Response::SearchSuccess(vec![n]);
            let retrieve_response = Response::RetrieveSuccess(s.clone());
            assert_eq!(
                Response::from_bytes(&pub_response.to_bytes()[..]).unwrap(),
                pub_response
            );
            assert_eq!(
                Response::from_bytes(&search_response.to_bytes()[..]).unwrap(),
                search_response
            );
            assert_eq!(
                Response::from_bytes(&retrieve_response.to_bytes()[..]).unwrap(),
                retrieve_response
            );
        }
        quickcheck(round_trip_response as fn(String, usize));
    }
}

// ============================ ARGUMENTS ============================

// graded manually

// ============================ CLIENT + Server============================

mod integration {
    use super::*;
    use ngram::message::*;
    use ngram::{client, server};
    use std::fs;
    use std::sync::{Arc, Mutex};
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    fn start_server(port: u16) -> (Arc<server::Server>, JoinHandle<()>) {
        let server = Arc::new(server::Server::new());
        let handle = thread::spawn({
            let server = Arc::clone(&server);
            move || server.run(port)
        });
        thread::sleep(Duration::from_millis(500));
        (server, handle)
    }

    #[test]
    fn test_start_stop_server_5() {
        let port = 7880;
        let (server, _handle) = start_server(port);
        server.stop();
    }

    #[test]
    fn test_publish_5() {
        let port = 7881;
        let (server, _handle) = start_server(port);

        let client = client::Client::new("127.0.0.1", port);
        let response = client.publish_from_path("data/austen-emma.txt");
        assert!(matches!(response, Some(Response::PublishSuccess(_))));
        server.stop();
    }

    #[test]
    fn test_search_empty_5() {
        let port = 7882;
        let (server, _handle) = start_server(port);

        let client = client::Client::new("127.0.0.1", port);
        let response = client.search("a");
        assert_eq!(response, Some(Response::SearchSuccess(vec![])));
        server.stop();
    }

    #[test]
    fn test_search_inserted_5() {
        let port = 7883;
        let (server, _handle) = start_server(port);

        let client = client::Client::new("127.0.0.1", port);

        let response = client.publish_from_path("data/austen-emma.txt");
        let id = match response {
            Some(Response::PublishSuccess(id)) => id,
            _ => panic!("Failed to publish data/austen-emma.txt"),
        };
        let response = client.search("the");
        assert_eq!(response, Some(Response::SearchSuccess(vec![id])));
        server.stop();
    }

    #[test]
    fn test_search_multiple_5() {
        let port = 7884;
        let (server, _handle) = start_server(port);

        let client = client::Client::new("127.0.0.1", port);
        let id1 = match client.publish_from_path("data/austen-emma.txt") {
            Some(Response::PublishSuccess(id)) => id,
            _ => panic!("Failed to publish data/austen-emma.txt"),
        };
        let id2 = match client.publish_from_path("data/austen-persuasion.txt") {
            Some(Response::PublishSuccess(id)) => id,
            _ => panic!("Failed to publish data/austen-persuasion.txt"),
        };

        let response = client.search("little");
        if let Some(Response::SearchSuccess(ids)) = response {
            assert_eq!(ids.len(), 2);
            assert!(ids.contains(&id1));
            assert!(ids.contains(&id2));
        } else {
            panic!("Failed to search for 'little'");
        }
        server.stop();
    }

    #[test]
    fn test_search_distractor_5() {
        let port = 7885;
        let (server, _handle) = start_server(port);

        let client = client::Client::new("127.0.0.1", port);
        let _id1 = match client.publish_from_path("data/austen-persuasion.txt") {
            Some(Response::PublishSuccess(id)) => id,
            _ => panic!("Failed to publish data/austen-persuasion.txt"),
        };
        let id2 = match client.publish_from_path("data/austen-emma.txt") {
            Some(Response::PublishSuccess(id)) => id,
            _ => panic!("Failed to publish data/austen-emma.txt"),
        };

        let response = client.search("ceased");
        assert_eq!(response, Some(Response::SearchSuccess(vec![id2])));
        server.stop();
    }

    #[test]
    fn test_retrieve_5() {
        let port = 7886;
        let (server, _handle) = start_server(port);

        let client = client::Client::new("127.0.0.1", port);
        let id = match client.publish_from_path("data/austen-emma.txt") {
            Some(Response::PublishSuccess(id)) => id,
            _ => panic!("Failed to publish data/austen-emma.txt"),
        };
        let doc = std::fs::read_to_string("data/austen-emma.txt").unwrap();
        let response = client.retrieve(id);
        assert_eq!(response, Some(Response::RetrieveSuccess(doc)));
        server.stop();
    }

    #[test]
    fn test_server_stress_test_10() {
        let port = 7889;
        let (server, _handle) = start_server(port);

        let paths = vec![
            "data/austen-emma.txt",
            "data/austen-persuasion.txt",
            "data/austen-sense.txt",
            "data/bible-kjv.txt",
            "data/blake-poems.txt",
            "data/bryant-stories.txt",
            "data/burgess-busterbrown.txt",
            "data/carroll-alice.txt",
            "data/chesterton-ball.txt",
            "data/chesterton-brown.txt",
            "data/chesterton-thursday.txt",
            "data/edgeworth-parents.txt",
            "data/melville-moby_dick.txt",
            "data/milton-paradise.txt",
            "data/shakespeare-caesar.txt",
            "data/shakespeare-hamlet.txt",
            "data/shakespeare-macbeth.txt",
            "data/whitman-leaves.txt",
        ];

        let queue = Arc::new(Mutex::new(paths));
        println!("Adding docs...");
        let now = std::time::Instant::now();
        let handles = (0..THREADS)
            .map(|i| {
                thread::spawn({
                    let queue = Arc::clone(&queue);
                    move || loop {
                        let client = client::Client::new("127.0.0.1", port);
                        let path = queue.lock().unwrap().pop().clone();
                        match path {
                            Some(path) => {
                                println!("Thread {}: processing {}", i, path);
                                client.publish_from_path(path);
                            }
                            None => return,
                        }
                    }
                })
            })
            .collect::<Vec<_>>();
        handles.into_iter().for_each(|h| h.join().unwrap());

        let words = fs::read_to_string("data/words.txt").unwrap();
        let words = words
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // Search docs sequentially
        let _now = std::time::Instant::now();
        let client = client::Client::new("127.0.0.1", port);
        for word in words.iter() {
            let response = client.search(word);
            assert!(matches!(response, Some(Response::SearchSuccess(_))));
        }
        // println!("Sequential search took {:?}", _now.elapsed);

        // Search docs in parallel
        let word_queue = Arc::new(Mutex::new(words));
        let _now = std::time::Instant::now();
        let handles = (0..THREADS)
            .map(|_| {
                thread::spawn({
                    let word_queue = Arc::clone(&word_queue);
                    let client = client::Client::new("127.0.0.1", port);
                    move || loop {
                        let word = word_queue.lock().unwrap().pop().clone();
                        match word {
                            Some(word) => {
                                let response = client.search(&word);
                                assert!(matches!(response, Some(Response::SearchSuccess(_))));
                                //println!("Found {} in {:?}", word, indices);
                            }
                            None => return,
                        }
                    }
                })
            })
            .collect::<Vec<_>>();
        handles.into_iter().for_each(|h| h.join().unwrap());
        // println!("Parallel search took {:?}", _now.elapsed());

        server.stop();
    }
}
