#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{convert::TryFrom, thread, time::Duration};

use iroha_futures::FuturePollTelemetry;
use iroha_logger::config::LoggerConfiguration;
use tokio::task;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

#[iroha_futures::telemetry_future]
async fn sleep(times: Vec<Duration>) -> i32 {
    for time in times {
        thread::sleep(time);
        task::yield_now().await;
    }
    // Just random result
    10
}

fn almost_equal(a: Duration, b: Duration) -> bool {
    (a - b) < (b / 9)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_sleep() {
    if cfg!(not(feature = "telemetry")) {
        return;
    }

    let sleep_times = vec![
        Duration::from_nanos(100_000_000),
        Duration::from_nanos(70_000_000),
        Duration::from_nanos(80_000_000),
    ];

    let (_, telemetry_future) = iroha_logger::init(&LoggerConfiguration::default())
        .unwrap()
        .unwrap();
    assert_eq!(sleep(sleep_times.clone()).await, 10);
    let telemetry = ReceiverStream::new(telemetry_future)
        .map(FuturePollTelemetry::try_from)
        .filter_map(Result::ok)
        .take(3)
        .collect::<Vec<_>>()
        .await;
    assert_eq!(telemetry.len(), 3);

    let id = telemetry[0].id;
    let times = telemetry.iter().map(|telemetry| telemetry.duration);

    assert!(telemetry
        .iter()
        .all(|telemetry| telemetry.name == "basic::sleep"));
    assert!(telemetry.iter().all(|telemetry| telemetry.id == id));
    assert!(times.zip(sleep_times).all(|(a, b)| almost_equal(a, b)));
}