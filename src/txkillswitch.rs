use std::process::{exit, Command};
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};

const UPTIME_SRC_FILEPATH: &'static str = "/proc/uptime";

fn arg_error(error_msg: &str) {
    eprintln!("{}", error_msg);
    eprintln!("Parameters: interface, rate_limit, check_interval, report_interval, status_ensure_interval");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 6 {
        arg_error("Expected exactly 5 arguments"); // TODO: Make parameters optional, named
        exit(1)
    }

    // TODO: Validation
    let bytes_src_filepath = "/sys/class/net/".to_owned() + &args[1] + "/statistics/tx_bytes";
    let rate_limit: u64 = args[2].parse().unwrap_or(0);
    let check_interval: u64 = args[3].parse().unwrap_or(0);
    let report_interval: u64 = args[4].parse().unwrap_or(0);
    let status_ensure_interval: u64 = args[5].parse().unwrap_or(0);

    let pause: Duration = Duration::from_secs(check_interval);
    let mut report_counter: u64 = report_interval;
    let mut state_counter: u64 = 0;
    let mut was_breached: bool = false;

    loop {
        let rate: u64 = get_current_rate(&bytes_src_filepath);
        let is_breached: bool = rate > rate_limit;

        if is_breached != was_breached {
            println!("State changed at rate: {}", rate);
            report_counter = 0;
        }

        if is_breached != was_breached || state_counter >= status_ensure_interval {
            state_counter = 0;

            assure_service_status(is_breached);
            was_breached = is_breached;
        }

        if report_counter >= report_interval {
            report_counter = 0;

            println!("Current rate: {}", rate);
        }

        report_counter += 1;
        state_counter += 1;

        sleep(pause);
    }
}

fn assure_service_status(is_breached: bool) {
    println!("Assuring service state");

    if !is_breached {
        Command::new("systemctl")
            .arg("start")
            .arg("httpd")
            .output()
            .expect("Failed to execute process"); // TODO: Report failure
    } else {
        Command::new("systemctl")
            .arg("stop")
            .arg("httpd")
            .output()
            .expect("Failed to execute process"); // TODO: Report failure
    }
}

fn get_current_rate(bytes_src_filepath: &str) -> u64 {
    let bytes_str = fs::read_to_string(bytes_src_filepath).expect("error reading bytes stats"); // TODO: Don't panic

    let bytes: u64 = bytes_str.trim_end().parse().unwrap_or(std::u64::MAX); // TODO: Report invalid values

    let uptime_str: String =
        fs::read_to_string(UPTIME_SRC_FILEPATH).expect("error reading bytes stats"); // TODO: Don't panic

    let uptime_seconds: u64 = uptime_str
        .split(".")
        .next()
        .unwrap_or("1")
        .parse()
        .unwrap_or(1); // TODO: Report invalid values

    return bytes / uptime_seconds;
}
