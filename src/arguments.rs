use std::env::Args;
use std::ops::Add;

use anyhow::{ensure, Result};

pub struct Config {
    pub bytes_src_filepath: String,
    pub rate_limit: u64,
    pub check_interval: u64,
    pub report_interval: u64,
    pub status_ensure_interval: u64,
}

// TODO: Make parameters optional, named
// TODO: Validation
pub fn parse_args<'a>(args: Args) -> Result<Config> {
    let args: Vec<String> = args.collect();

    ensure!(args.len() == 6, "Expected exactly 5 arguments: interface, rate_limit, check_interval, report_interval, status_ensure_interval");

    let bytes_src_filepath = "/sys/class/net/".to_string()
        .add(args[1].as_str())
        .add("/statistics/tx_bytes");
    let rate_limit: u64 = args[2].parse().unwrap_or(0);
    let check_interval: u64 = args[3].parse().unwrap_or(0);
    let report_interval: u64 = args[4].parse().unwrap_or(0);
    let status_ensure_interval: u64 = args[5].parse().unwrap_or(0);

    return Ok(Config { bytes_src_filepath, rate_limit, check_interval, report_interval, status_ensure_interval });
}
