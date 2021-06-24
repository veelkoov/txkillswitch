use std::env::Args;
use std::fs;

use anyhow::{anyhow, Context, Result};
use getopts::{Matches, Options};

pub struct Config {
    pub rx_info_file_path: String,
    pub tx_info_file_path: String,
    pub rate_limit: u64,
    pub check_interval: u64,
    pub report_interval: u64,
    pub status_ensure_interval: u64,
    pub count_tx: bool,
    pub count_rx: bool,
}

const DEFAULT_CHECK_INTERVAL: u64 = 60;
const DEFAULT_REPORT_EVERY: u64 = 10;
const DEFAULT_ENSURE_EVERY: u64 = 5;

const OPT_INTERFACE: &'static str = "interface";
const OPT_RATE_LIMIT: &'static str = "rate-limit";
const OPT_CHECK_INTERVAL: &'static str = "check-interval";
const OPT_REPORT_EVERY: &'static str = "report-every";
const OPT_ENSURE_EVERY: &'static str = "ensure-every";
const OPT_TX: &'static str = "tx";
const OPT_RX: &'static str = "rx";

const USAGE: &'static str = "Usage: txkillswitch -i INTERFACE -l BYTES";

pub fn parse_args(args: Args) -> Result<Config> {
    let opt_cfg = get_options();

    let result = opt_cfg.parse(args);
    let options = result.with_context(|| format!("wrong arguments\n{}", opt_cfg.usage(USAGE)))?;

    let count_tx = opt_bool(&options, OPT_TX);
    let count_rx = opt_bool(&options, OPT_RX);

    if !count_rx && !count_tx {
        return Err(anyhow!("at least one of --{} and --{} are required\n{}", OPT_RX, OPT_TX, opt_cfg.usage(USAGE)))
    }

    let interface = options.opt_str(OPT_INTERFACE).unwrap();
    let rx_info_file_path = get_checked_info_file_path(&interface, "rx", count_rx )?;
    let tx_info_file_path = get_checked_info_file_path(&interface, "tx", count_tx )?;
    let rate_limit = req_u64(&options, OPT_RATE_LIMIT)?;
    let check_interval = opt_u64(&options, OPT_CHECK_INTERVAL, DEFAULT_CHECK_INTERVAL)?;
    let report_interval = opt_u64(&options, OPT_REPORT_EVERY, DEFAULT_REPORT_EVERY)?;
    let status_ensure_interval = opt_u64(&options, OPT_ENSURE_EVERY, DEFAULT_ENSURE_EVERY)?;

    let free = &options.free[1..];

    match free.is_empty() {
        false => Err(anyhow!("unexpected arguments: {:?}\n{}", free, opt_cfg.usage(USAGE))),
        true => Ok(Config {
            rx_info_file_path,
            tx_info_file_path,
            rate_limit,
            check_interval,
            report_interval,
            status_ensure_interval,
            count_tx,
            count_rx,
        }),
    }
}

fn get_options() -> Options {
    let mut result = Options::new();

    result.reqopt("i", OPT_INTERFACE,
                   "Monitored network interface name", "INTERFACE");

    result.reqopt("l", OPT_RATE_LIMIT,
                   "Rate limit in bytes", "BYTES");

    result.optopt("c", OPT_CHECK_INTERVAL, format!(
        "Interval between checks in seconds (default {})", DEFAULT_CHECK_INTERVAL)
        .as_str(), "SECONDS");

    result.optopt("r", OPT_REPORT_EVERY, format!(
        "Echo current rate every N intervals (default {})", DEFAULT_REPORT_EVERY)
        .as_str(), "INTERVALS");

    result.optopt("e", OPT_ENSURE_EVERY, format!(
        "Re-run command every N intervals (default {})", DEFAULT_ENSURE_EVERY)
        .as_str(), "INTERVALS");

    result.optflag("", OPT_TX, "Count transmitted data");
    result.optflag("", OPT_RX, "Count received data");

    result
}

fn get_checked_info_file_path(interface: &str, tx_or_rx: &str, used: bool) -> Result<String> {
    if !used {
        return Ok(String::from(""));
    }

    let info_file_path = format!("/sys/class/net/{}/statistics/{}_bytes", interface, tx_or_rx);

    // Test if we can read / interface exists
    fs::read_to_string(info_file_path.as_str()).with_context(|| format!("could not read `{}`", info_file_path))?;

    Ok(info_file_path)
}

fn req_u64(matches: &Matches, option_name: &str) -> Result<u64> {
    let val = matches.opt_str(option_name).unwrap();

    val.parse().with_context(|| format!("couldn't parse '{}' in option '{}'", val, option_name))
}

fn opt_u64(matches: &Matches, option_name: &str, default: u64) -> Result<u64> {
    let val = matches.opt_str(option_name).unwrap_or(default.to_string());

    val.parse().with_context(|| format!("couldn't parse '{}' in option '{}'", val, option_name))
}

fn opt_bool(matches: &Matches, option_name: &str) -> bool {
    matches.opt_present(option_name)
}
