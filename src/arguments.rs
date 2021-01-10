use std::env::Args;
use std::fs;
use std::ops::Add;

use anyhow::{bail, Context, Result};
use getopts::{Matches, Options};

pub struct Config {
    pub bytes_src_filepath: String,
    pub rate_limit: u64,
    pub check_interval: u64,
    pub report_interval: u64,
    pub status_ensure_interval: u64,
}

const OPT_INTERFACE: &'static str = "interface";
const OPT_RATE_LIMIT: &'static str = "rate-limit";
const OPT_CHECK_INTERVAL: &'static str = "check-interval";
const OPT_REPORT_EVERY: &'static str = "report-every";
const OPT_ENSURE_EVERY: &'static str = "ensure-every";

const USAGE: &'static str = "Usage: txkillswitch -i INTERFACE -l BYTES";

pub fn parse_args<'a>(args: Args) -> Result<Config> {
    let mut opt_cfg = Options::new();

    opt_cfg.reqopt("i", OPT_INTERFACE, "Monitored network interface name", "INTERFACE");
    opt_cfg.reqopt("l", OPT_RATE_LIMIT, "Rate limit in bytes", "BYTES");
    opt_cfg.optopt("c", OPT_CHECK_INTERVAL, "Interval between checks in seconds", "SECONDS");
    opt_cfg.optopt("r", OPT_REPORT_EVERY, "Echo current rate every N intervals", "INTERVALS");
    opt_cfg.optopt("e", OPT_ENSURE_EVERY, "Re-run command every N intervals", "INTERVALS");

    let result = opt_cfg.parse(args);
    let options = result.with_context(|| format!("wrong arguments\n{}", opt_cfg.usage(USAGE)))?;

    let bytes_src_filepath = get_checked_bytes_path(&options)?;
    let rate_limit: u64 = req_u64(&options, OPT_RATE_LIMIT)?;
    let check_interval: u64 = opt_u64(&options, OPT_CHECK_INTERVAL, 60)?;
    let report_interval: u64 = opt_u64(&options, OPT_REPORT_EVERY, 10)?;
    let status_ensure_interval: u64 = opt_u64(&options, OPT_ENSURE_EVERY, 5)?;

    let free = &options.free[1..];

    if !free.is_empty() {
        bail!(format!("unexpected arguments: {:?}\n{}", free, opt_cfg.usage(USAGE)));
    }

    return Ok(Config { bytes_src_filepath, rate_limit, check_interval, report_interval, status_ensure_interval });
}

fn get_checked_bytes_path(options: &Matches) -> Result<String> {
    let bytes_src_filepath = "/sys/class/net/".to_string()
        .add(options.opt_str(OPT_INTERFACE).unwrap().as_str())
        .add("/statistics/tx_bytes");

    // Test if we can read / interface exists
    fs::read_to_string(bytes_src_filepath.as_str())
        .with_context(|| format!("could not read `{}`", bytes_src_filepath))?;

    return Ok(bytes_src_filepath);
}

fn req_u64(matches: &Matches, option_name: &str) -> Result<u64> {
    let val = matches.opt_str(option_name).unwrap();

    val.parse().with_context(|| format!("couldn't parse '{}' in option '{}'", val, option_name))
}

fn opt_u64(matches: &Matches, option_name: &str, default: u64) -> Result<u64> {
    let val = matches.opt_str(option_name).unwrap_or(default.to_string());

    val.parse().with_context(|| format!("couldn't parse '{}' in option '{}'", val, option_name))
}
