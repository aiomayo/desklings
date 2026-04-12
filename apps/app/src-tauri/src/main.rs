#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    match desklings_lib::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fatal: {}", desklings_lib::error::format_chain(&e));
            ExitCode::FAILURE
        }
    }
}
