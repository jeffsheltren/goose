use httpmock::Method::{GET, POST};
use httpmock::{mock, with_mock_server};

mod common;

use goose::prelude::*;

const INDEX_PATH: &str = "/";
const SETUP_PATH: &str = "/setup";
const TEARDOWN_PATH: &str = "/teardown";

pub async fn setup(client: &GooseClient) {
    let _response = client.post(SETUP_PATH, "setting up load test").await;
}

pub async fn teardown(client: &GooseClient) {
    let _response = client
        .post(TEARDOWN_PATH, "cleaning up after load test")
        .await;
}

pub async fn get_index(client: &GooseClient) {
    let _response = client.get(INDEX_PATH).await;
}

/// Test test_start alone.
#[test]
#[with_mock_server]
fn test_start() {
    let mock_setup = mock(POST, SETUP_PATH).return_status(201).create();
    let mock_teardown = mock(POST, TEARDOWN_PATH).return_status(205).create();
    let mock_index = mock(GET, INDEX_PATH).return_status(200).create();

    crate::GooseAttack::initialize_with_config(common::build_configuration())
        .setup()
        .test_start(task!(setup))
        .register_taskset(taskset!("LoadTest").register_task(task!(get_index).set_weight(9)))
        .execute();

    let called_setup = mock_setup.times_called();
    let called_index = mock_index.times_called();
    let called_teardown = mock_teardown.times_called();

    // Confirm the load test ran.
    assert_ne!(called_index, 0);

    // Confirm we ran setup one time.
    assert_eq!(called_setup, 1);

    // Confirm we did not run the teardown.
    assert_eq!(called_teardown, 0);
}

/// Test test_stop alone.
#[test]
#[with_mock_server]
fn test_stop() {
    let mock_setup = mock(POST, SETUP_PATH).return_status(201).create();
    let mock_teardown = mock(POST, TEARDOWN_PATH).return_status(205).create();
    let mock_index = mock(GET, INDEX_PATH).return_status(200).create();

    crate::GooseAttack::initialize_with_config(common::build_configuration())
        .setup()
        .test_stop(task!(teardown))
        .register_taskset(taskset!("LoadTest").register_task(task!(get_index).set_weight(9)))
        .execute();

    let called_setup = mock_setup.times_called();
    let called_index = mock_index.times_called();
    let called_teardown = mock_teardown.times_called();

    // Confirm the load test ran.
    assert_ne!(called_index, 0);

    // Confirm we did not run setup.
    assert_eq!(called_setup, 0);

    // Confirm we ran the teardown 1 time.
    assert_eq!(called_teardown, 1);
}

#[test]
#[with_mock_server]
fn test_setup_teardown() {
    let mock_setup = mock(POST, SETUP_PATH).return_status(201).create();
    let mock_teardown = mock(POST, TEARDOWN_PATH).return_status(205).create();
    let mock_index = mock(GET, INDEX_PATH).return_status(200).create();

    let mut configuration = common::build_configuration();
    // Launch several client threads, confirm we still only setup and teardown one time.
    configuration.clients = Some(5);
    configuration.hatch_rate = 5;

    crate::GooseAttack::initialize_with_config(configuration)
        .setup()
        .test_start(task!(setup))
        .register_taskset(taskset!("LoadTest").register_task(task!(get_index).set_weight(9)))
        .test_stop(task!(teardown))
        .execute();

    let called_setup = mock_setup.times_called();
    let called_index = mock_index.times_called();
    let called_teardown = mock_teardown.times_called();

    // Confirm the load test ran.
    assert_ne!(called_index, 0);

    // Confirm we ran setup one time.
    assert_eq!(called_setup, 1);

    // Confirm we ran teardown one time.
    assert_eq!(called_teardown, 1);
}
