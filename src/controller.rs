use std::{io, process};
use std::io::Write;

use anyhow::{ensure, Result};

struct Command {
    whole_cmdline: String,
    executable: String,
    arguments: Vec<String>,
}

impl Command {
    fn new(whitespace_joined: String) -> Result<Command> {
        let parts: Vec<String> = whitespace_joined.split_ascii_whitespace()
            .map(|str| str.to_string()).collect();

        ensure!(!parts.is_empty(), "failed to parse command - empty or whitespace only");

        Ok(Command {
            whole_cmdline: whitespace_joined.clone(),
            executable: parts[0].clone(),
            arguments: parts[1..].to_vec(),
        })
    }
}

pub struct Controller {
    compliant: Command,
    breaching: Command,
}

pub trait ControllerT {
    fn new_for_systemd_service(service_name: &str) -> Result<Controller>;
    fn run_for(&self, is_breaching: bool) -> ();
}

impl ControllerT for Controller {
    fn new_for_systemd_service(service_name: &str) -> Result<Controller> {
        Ok(Controller {
            breaching: Command::new(format!("sudo systemctl stop {}", service_name))?,
            compliant: Command::new(format!("sudo systemctl start {}", service_name))?,
        })
    }

    fn run_for(&self, is_breaching: bool) -> () {
        let command = if is_breaching { &self.breaching } else { &self.compliant };

        let result = process::Command::new(&command.executable)
            .args(&command.arguments)
            .output();

        match result {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("Command '{}' failed: {}", &command.whole_cmdline, output.status);
                    io::stdout().write_all(&output.stdout).unwrap();
                    io::stderr().write_all(&output.stderr).unwrap();
                }
            }
            Err(error) => {
                eprintln!("Failed to execute command '{}': {}", &command.whole_cmdline, error);
            }
        }
    }
}
