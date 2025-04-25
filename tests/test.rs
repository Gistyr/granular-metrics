// granular-metrics/tests/test.rs

// cargo test test_one -- --nocapture
// cargo test test_two --features http -- --nocapture

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
enum Keys {
    One,
    Two,
    Three,
    Four,
}

#[cfg(not(feature = "http"))]
#[tokio::test]
async fn test_one() {
    let log_settings: better_logger::LoggerSettings = better_logger::LoggerSettings {
        terminal_logs: true,
        terminal_log_lvl: "debug".to_string(),
        file_logs: false,
        file_log_lvl: "trace".to_string(),
        log_file_path: "null".to_string(),
        debug_extra: false,
        async_logging: false,
    };

    better_logger::logger::init(log_settings);
    
    granular_metrics::init::<Keys>();

    let (one, two, three, four) = tokio::join!(
        test_1(),
        test_2(),
        test_3(),
        test_4(),
    );
    assert!(one && two && three && four);

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    let snapshot = granular_metrics::fetch::<Keys>();

    better_logger::logger::info!("{:?}", &snapshot);

    assert_eq!(snapshot.per_key.len(), 4);

    let total_one: u64 = 1000;
    let total_two: u64 = 100;
    let total_three: u64 = 500;
    let total_four: u64 = 10;
    let sum_total: u64   = total_one + total_two + total_three + total_four;

    let avg_one: u64 = total_one / 60; 
    let avg_two: u64 = total_two / 60; 
    let avg_three: u64 = total_three / 60; 
    let avg_four: u64 = total_four / 60; 

    assert_eq!(snapshot.per_key.get(&Keys::One), Some(&(avg_one, total_one)));
    assert_eq!(snapshot.per_key.get(&Keys::Two), Some(&(avg_two, total_two)));
    assert_eq!(snapshot.per_key.get(&Keys::Three), Some(&(avg_three, total_three)));
    assert_eq!(snapshot.per_key.get(&Keys::Four), Some(&(avg_four, total_four)));

    let (avg_total, total) = snapshot.total;
    assert_eq!(total, sum_total);
    assert_eq!(avg_total, sum_total / 60);
}

#[cfg(feature = "http")]
#[tokio::test]
async fn test_two() {
    let log_settings: better_logger::LoggerSettings = better_logger::LoggerSettings {
        terminal_logs: true,
        terminal_log_lvl: "debug".to_string(),
        file_logs: false,
        file_log_lvl: "trace".to_string(),
        log_file_path: "null".to_string(),
        debug_extra: false,
        async_logging: false,
    };

    better_logger::logger::init(log_settings);
    
    granular_metrics::init::<Keys>("127.0.0.1", "8080", "metrics", "all");

    let (one, two, three, four) = tokio::join!(
        test_1(),
        test_2(),
        test_3(),
        test_4(),
    );
    assert!(one && two && three && four);

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    let snapshot = granular_metrics::fetch::<Keys>();

    better_logger::logger::info!("{:?}", &snapshot);

    assert_eq!(snapshot.per_key.len(), 4);

    let total_one: u64 = 1000;
    let total_two: u64 = 100;
    let total_three: u64 = 500;
    let total_four: u64 = 10;
    let sum_total: u64   = total_one + total_two + total_three + total_four;

    let avg_one: u64 = total_one / 60; 
    let avg_two: u64 = total_two / 60; 
    let avg_three: u64 = total_three / 60; 
    let avg_four: u64 = total_four / 60; 

    assert_eq!(snapshot.per_key.get(&Keys::One), Some(&(avg_one, total_one)));
    assert_eq!(snapshot.per_key.get(&Keys::Two), Some(&(avg_two, total_two)));
    assert_eq!(snapshot.per_key.get(&Keys::Three), Some(&(avg_three, total_three)));
    assert_eq!(snapshot.per_key.get(&Keys::Four), Some(&(avg_four, total_four)));

    let (avg_total, total) = snapshot.total;
    assert_eq!(total, sum_total);
    assert_eq!(avg_total, sum_total / 60);
}

async fn test_1() -> bool {
    let mut count: u16 = 0;
    
    while count < 1000 {
        granular_metrics::increment(Keys::One);

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        count += 1;
    }

    return true;
}

async fn test_2() -> bool {
    use Keys::*;
    use granular_metrics::increment;
    
    let mut count: u8 = 0;
    
    while count < 100 {
        increment(Two);

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        count += 1;
    }

    return true;
}

async fn test_3() -> bool {
    let mut count: u16 = 0;
    
    while count < 500 {
        granular_metrics::increment(Keys::Three);

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        count += 1;
    }

    return true;
}

async fn test_4() -> bool {
    let mut count: u8 = 0;
    
    while count < 10 {
        granular_metrics::increment(Keys::Four);

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        count += 1;
    }

    return true;
}