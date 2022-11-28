mod common;
use common::tests::{gen_requests_failed_load, gen_requests_set, IpVersion, ServerWrapper};
use hash_delivery_network::client::Client;
use serial_test::serial;
use std::{thread::JoinHandle, time::Instant};

const BENCH_REQUESTS_NUM: usize = 50_000;

#[test]
#[serial]
fn bench_clients_1() {
    bench_clients(1, BENCH_REQUESTS_NUM);
}

#[test]
#[serial]
fn bench_clients_2() {
    bench_clients(2, BENCH_REQUESTS_NUM);
}

#[test]
#[serial]
fn bench_clients_4() {
    bench_clients(4, BENCH_REQUESTS_NUM);
}

#[test]
#[serial]
fn bench_clients_8() {
    bench_clients(8, BENCH_REQUESTS_NUM);
}

#[test]
#[serial]
fn bench_clients_16() {
    bench_clients(16, BENCH_REQUESTS_NUM);
}

#[test]
#[serial]
fn bench_clients_32() {
    bench_clients(32, BENCH_REQUESTS_NUM);
}

/// Measures server performance in storing, successful and failed loading hashes scenarios
fn bench_clients(clients_num: usize, requests_num: usize) {
    let server = ServerWrapper::start(IpVersion::V4).unwrap();
    let mut clients: Vec<Client> = (0..clients_num)
        .map(|id| Client::start(id as u64, server.addr).unwrap())
        .collect();
    for client in clients.iter_mut() {
        let _greeting = client.read().unwrap();
    }

    // (Vec<u8>, Vec<u8>) -> requests and responses
    // Vec<$uppper$> -> for a client
    // Vec<$upper$> -> for all clients
    let (reqs_sets_store, reqs_sets_succ_load): (
        Vec<Vec<(Vec<u8>, Vec<u8>)>>,
        Vec<Vec<(Vec<u8>, Vec<u8>)>>,
    ) = (0..clients_num)
        .map(|i| {
            gen_requests_set(
                i * requests_num / clients_num,
                (i + 1) * requests_num / clients_num,
            )
        })
        .unzip();
    {
        let mut reqs_sets_store = reqs_sets_store;
        let timer = Instant::now();
        let threads: Vec<JoinHandle<()>> = (0..clients_num)
            .map(|i| {
                let mut client = clients[i].clone();
                let reqs_store = reqs_sets_store.pop().unwrap();
                std::thread::spawn(move || {
                    for (req, resp) in reqs_store.iter() {
                        client.write(req).unwrap();
                        assert_eq!(resp, &client.read().unwrap());
                    }
                })
            })
            .collect();
        for thread in threads.into_iter() {
            thread.join().unwrap();
        }
        println!(
            "\nStore requests:                {} reqs/s",
            (requests_num as f64 / timer.elapsed().as_secs_f64()) as u64
        );
    }
    {
        let mut reqs_sets_succ_load = reqs_sets_succ_load;
        let timer = Instant::now();
        let threads: Vec<JoinHandle<()>> = (0..clients_num)
            .map(|i| {
                let mut client = clients[i].clone();
                let reqs_load = reqs_sets_succ_load.pop().unwrap();
                std::thread::spawn(move || {
                    for (req, resp) in reqs_load.iter() {
                        client.write(req).unwrap();
                        assert_eq!(resp, &client.read().unwrap());
                    }
                })
            })
            .collect();
        for thread in threads.into_iter() {
            thread.join().unwrap();
        }
        println!(
            "Successful load requests:      {} reqs/s",
            (requests_num as f64 / timer.elapsed().as_secs_f64()) as u64
        );
    }
    let mut reqs_sets_fail_load: Vec<Vec<(Vec<u8>, Vec<u8>)>> = (0..clients_num)
        .map(|i| {
            gen_requests_failed_load(
                i * requests_num / clients_num,
                (i + 1) * requests_num / clients_num,
            )
        })
        .collect();
    {
        let timer = Instant::now();
        let threads: Vec<JoinHandle<()>> = (0..clients_num)
            .into_iter()
            .map(|i| {
                let mut client = clients[i].clone();
                let reqs_load = reqs_sets_fail_load.pop().unwrap();
                std::thread::spawn(move || {
                    for (req, resp) in reqs_load.iter() {
                        client.write(req).unwrap();
                        let ret = client.read().unwrap();
                        assert_eq!(resp, &ret);
                    }
                })
            })
            .collect();
        for thread in threads.into_iter() {
            thread.join().unwrap();
        }
        println!(
            "Failed load requests:          {} reqs/s",
            (requests_num as f64 / timer.elapsed().as_secs_f64()) as u64
        );
    }
}
