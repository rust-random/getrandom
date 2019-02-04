extern crate getrandom;

use getrandom::getrandom;

#[test]
fn test_diff() {
    let mut v1 = [0u8; 1000];
    getrandom(&mut v1).unwrap();

    let mut v2 = [0u8; 1000];
    getrandom(&mut v2).unwrap();

    let mut n_diff_bits = 0;
    for i in 0..v1.len() {
        n_diff_bits += (v1[i] ^ v2[i]).count_ones();
    }

    // Check at least 1 bit per byte differs. p(failure) < 1e-1000 with random input.
    assert!(n_diff_bits >= v1.len() as u32);
}

#[test]
fn test_huge() {
    let mut huge = [0u8; 100_000];
    getrandom(&mut huge).unwrap();
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
#[test]
fn test_os_rng_tasks() {
    use std::sync::mpsc::channel;
    use std::thread;

    let mut txs = vec!();
    for _ in 0..20 {
        let (tx, rx) = channel();
        txs.push(tx);

        thread::spawn(move|| {
            // wait until all the tasks are ready to go.
            rx.recv().unwrap();
            let mut v = [0u8; 1000];

            for _ in 0..100 {
                getrandom(&mut v).unwrap();
                thread::yield_now();
            }
        });
    }

    // start all the tasks
    for tx in txs.iter() {
        tx.send(()).unwrap();
    }
}
