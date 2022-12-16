mod common;
use common::tests::{Client, IpVersion, ServerWrapper};
use serial_test::serial;
use std::{collections::HashMap, thread::sleep, time::Duration};

fn are_json_equal(data1: &[u8], data2: &[u8]) -> bool {
    let map1: HashMap<String, String> =
        serde_json::from_str(&String::from_utf8(data1.to_vec()).unwrap()).unwrap();
    let map2: HashMap<String, String> =
        serde_json::from_str(&String::from_utf8(data2.to_vec()).unwrap()).unwrap();
    map1.len() == map2.len() && map1.keys().all(|k| map1.get(k) == map2.get(k))
}

#[test]
#[serial]
fn test_load_and_store() {
    let server = ServerWrapper::start(IpVersion::V4).unwrap();

    let mut client = Client::start(0, server.addr).unwrap();

    // test greeting
    {
        assert!(are_json_equal(
            b"{\
            \n\t\"student_name\": \"das67333\"\
            \n}",
            &client.read().unwrap()
        ));
    }
    // test failed load
    {
        let message = b"{\
        \n\t\"request_type\": \"load\",\
        \n\t\"key\": \"some_key\"\
        \n}";
        client.write(message).unwrap();
        assert!(are_json_equal(
            b"{\
            \n\t\"response_status\": \"key not found\"\
            \n}",
            &client.read().unwrap()
        ));
    }
    // test store
    {
        let message = b"{\
        \n\t\"request_type\": \"store\",\
        \n\t\"key\": \"some_key\",\
        \n\t\"hash\": \"0b672dd94fd3da6a8d404b66ee3f0c83\"\
        \n}";
        client.write(message).unwrap();
        assert!(are_json_equal(
            b"{\
            \n\t\"response_status\": \"success\"\
            \n}",
            &client.read().unwrap()
        ));
    }
    // test successful load
    {
        let message = b"{\
        \n\t\"request_type\": \"load\",\
        \n\t\"key\": \"some_key\"\
        \n}";
        client.write(message).unwrap();
        assert!(are_json_equal(
            b"{\
            \n\t\"response_status\": \"success\",\
            \n\t\"requested_key\": \"some_key\",\
            \n\t\"requested_hash\": \"0b672dd94fd3da6a8d404b66ee3f0c83\"\
            \n}",
            &client.read().unwrap()
        ));
    }
    client.shutdown();
    sleep(Duration::from_millis(10));
}

#[test]
#[serial]
fn test_shutdown() {
    let mut server = ServerWrapper::start(IpVersion::V4).unwrap();

    let mut client = Client::start(0, server.addr).unwrap();

    // greeting
    client.read().unwrap();

    let message = b"{\
    \n\t\"request_type\": \"shutdown\"\
    \n}";
    client.write(message).unwrap();
    sleep(Duration::from_millis(10));
    assert!(!server.is_alive());
}
