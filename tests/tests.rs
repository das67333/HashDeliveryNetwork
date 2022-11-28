mod common;
use common::tests::{IpVersion, ServerWrapper};
use hash_delivery_network::client::Client;
use serial_test::parallel;
use std::collections::HashMap;

fn are_json_equal(data1: &[u8], data2: &[u8]) -> bool {
    let map1: HashMap<String, String> =
        serde_json::from_str(&String::from_utf8(data1.to_vec()).unwrap()).unwrap();
    let map2: HashMap<String, String> =
        serde_json::from_str(&String::from_utf8(data2.to_vec()).unwrap()).unwrap();
    map1.len() == map2.len() && map1.keys().all(|k| map1.get(k) == map2.get(k))
}

#[test]
#[parallel]
fn test_ddos() {
    let server = ServerWrapper::start(IpVersion::V4).unwrap();

    let n = 100;
    let mut clients: Vec<Client> = (0..n)
        .filter_map(|i| Client::start(i, server.addr).ok())
        .collect();
    for client in clients.iter_mut() {
        let _greeting = client.read().unwrap();
        let message = format!("client {}: corrupted json", client.id);
        client.write(message.as_bytes()).unwrap();
    }
    for client in clients.iter_mut() {
        client.shutdown();
    }
}

#[test]
#[parallel]
fn test_store_and_load() {
    let server = ServerWrapper::start(IpVersion::V4).unwrap();

    let mut client = Client::start(0, server.addr).unwrap();

    // test greeting
    {
        assert!(are_json_equal(
            b"{\
            \n  \"student_name\": \"das67333\"\
            \n}",
            &client.read().unwrap()
        ));
    }
    // test failed load
    {
        let message = b"{\
        \n  \"request_type\": \"load\",\
        \n  \"key\": \"some_key\"\
        \n}";
        client.write(message).unwrap();
        assert!(are_json_equal(
            b"{\
            \n  \"response_status\": \"key not found\"\
            \n}",
            &client.read().unwrap()
        ));
    }
    // test store
    {
        let message = b"{\
        \n  \"request_type\": \"store\",\
        \n  \"key\": \"some_key\",\
        \n  \"hash\": \"0b672dd94fd3da6a8d404b66ee3f0c83\"\
        \n}";
        client.write(message).unwrap();
        assert!(are_json_equal(
            b"{\
            \n  \"response_status\": \"success\"\
            \n}",
            &client.read().unwrap()
        ));
    }
    // test successful load
    {
        let message = b"{\
                                \n  \"request_type\": \"load\",\
                                \n  \"key\": \"some_key\"\
                                \n}";
        client.write(message).unwrap();
        assert!(are_json_equal(
            b"{\
            \n  \"response_status\": \"success\",\
            \n  \"requested_key\": \"some_key\",\
            \n  \"requested_hash\": \"0b672dd94fd3da6a8d404b66ee3f0c83\"\
            \n}",
            &client.read().unwrap()
        ));
    }
    client.shutdown();
}
