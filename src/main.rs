use std::{env, fs};
use std::error::Error;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use arguments::Config;

mod arguments;

const UPTIME_SRC_FILEPATH: &'static str = "/proc/uptime";

fn main() -> Result<(), Box<dyn Error>> {
    let cfg: Config = arguments::parse_args(env::args())?;

    let pause: Duration = Duration::from_secs(cfg.check_interval);
    let mut report_counter: u64 = cfg.report_interval;
    let mut state_counter: u64 = 0;
    let mut was_breached: bool = false;

    loop {
        let rate: u64 = get_current_rate(cfg.bytes_src_filepath.as_str());
        let is_breached: bool = rate > cfg.rate_limit;

        if is_breached != was_breached {
            println!("State changed at rate: {}", rate);
            report_counter = 0;
        }

        if is_breached != was_breached || state_counter >= cfg.status_ensure_interval {
            state_counter = 0;

            assure_service_status(is_breached);
            was_breached = is_breached;
        }

        if report_counter >= cfg.report_interval {
            report_counter = 0;

            println!("Current rate: {}", rate);
        }

        report_counter += 1;
        state_counter += 1;

        sleep(pause);
    }
}

fn assure_service_status(is_breached: bool) {
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
