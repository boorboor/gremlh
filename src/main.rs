use anyhow::Result;
use colored::*;
use gremlh::{run, ui::print_summary, Args};
use std::process::ExitCode;

fn main() -> Result<ExitCode> {
    let args = Args::parse_and_finalize();

    match run(&args) {
        Ok(stats) => {
            if !args.is_stdin {
                Ok(print_summary(&stats, &args))
            } else if stats.get_total_gremlins() > 0 || stats.get_errors() > 0 {
                Ok(ExitCode::FAILURE)
            } else {
                Ok(ExitCode::SUCCESS)
            }
        }
        Err(e) => {
            eprintln!("{}: {:?}", "error".red().bold(), e);
            Ok(ExitCode::FAILURE)
        }
    }
}
