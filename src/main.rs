use std::env;
use std::error::Error;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use arguments::Config;
use rate::get_current_rate_safely;

mod arguments;
mod rate;

fn main() -> Result<(), Box<dyn Error>> {
    let cfg: Config = arguments::parse_args(env::args())?;

    let pause: Duration = Duration::from_secs(cfg.check_interval);
    let mut report_counter: u64 = cfg.report_interval;
    let mut state_counter: u64 = 0;
    let mut was_breached: bool = false;

    loop {
        let rate = get_current_rate_safely(cfg.bytes_src_filepath.as_str());
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
