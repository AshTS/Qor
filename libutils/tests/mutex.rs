use libutils::sync::Mutex;

#[test]
fn multiple_threads_counting() {
    let mutex = std::sync::Arc::new(Mutex::new(0isize));
    let mut threads = Vec::new();

    for _ in 0..10 {
        let this_mutex = mutex.clone();
        threads.push(std::thread::spawn(move || {
            for _ in 0..25 {
                *this_mutex.spin_lock() += 1
            }
        }));

        let this_mutex = mutex.clone();
        threads.push(std::thread::spawn(move || {
            for _ in 0..25 {
                *this_mutex.spin_lock() -= 1
            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }

    let lock = mutex.attempt_lock();
    assert!(lock.is_some());
    assert_eq!(*lock.unwrap(), 0);
}