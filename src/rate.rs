use std::error::Error;
use std::fs;

const UPTIME_SRC_FILEPATH: &'static str = "/proc/uptime";

pub fn get_current_rate_safely(bytes_src_filepath: &str) -> u64 {
    match get_current_rate(bytes_src_filepath) {
        Ok(rate) => rate,

        Err(error) => {
            eprintln!("Encountered rate reading problem: {:?}", error);

            u64::MAX
        }
    }
}

fn get_current_rate(bytes_src_filepath: &str) -> Result<u64, Box<dyn Error>> {
    let bytes: u64 = fs::read_to_string(bytes_src_filepath)?.trim_end().parse()?;

    let uptime_seconds = fs::read_to_string(UPTIME_SRC_FILEPATH)?;

    let uptime_seconds: u64 = uptime_seconds.split(".").next()
        .ok_or(format!("unexpected uptime format: `{}`", uptime_seconds))?
        .parse()?;

    return Ok(bytes / uptime_seconds);
}
