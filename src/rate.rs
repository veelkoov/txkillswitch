use std::error::Error;
use std::fs;
use anyhow::Context;

const UPTIME_SRC_FILEPATH: &'static str = "/proc/uptime";

pub fn get_current_rate_safely(rx_info_file_path: &str, tx_info_file_path: &str) -> u64 {
    match get_current_rate(rx_info_file_path, tx_info_file_path) {
        Ok(rate) => rate,

        Err(error) => {
            eprintln!("Encountered rate reading problem: {:?}", error);

            u64::MAX
        }
    }
}

fn get_current_rate(rx_info_file_path: &str, tx_info_file_path: &str) -> Result<u64, Box<dyn Error>> {
    let mut bytes: u64 = 0;

    if !rx_info_file_path.is_empty() {
        bytes += read_number_from_file(rx_info_file_path)?;
    }

    if !tx_info_file_path.is_empty() {
        bytes += read_number_from_file(tx_info_file_path)?;
    }

    let uptime_seconds = fs::read_to_string(UPTIME_SRC_FILEPATH)?;

    let uptime_seconds: u64 = uptime_seconds.split(".").next()
        .ok_or(format!("unexpected uptime format: `{}`", uptime_seconds))?
        .parse()?;

    return Ok(bytes / uptime_seconds);
}

fn read_number_from_file(file_path: &str) -> Result<u64, Box<dyn Error>> {
    let contents = fs::read_to_string(file_path)
        .with_context(|| format!("failed reading `{}`", file_path))?;

    let bytes = contents.trim_end().parse::<u64>()
        .with_context(|| format!("failed parsing `{}`", file_path))?;

    Ok(bytes)
}
