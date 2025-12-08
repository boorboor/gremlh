use crate::args::Args;
use crate::stats::ScanStats;
use colored::Colorize;
use std::process::ExitCode;
use std::sync::Arc;

pub fn print_summary(stats: &Arc<ScanStats>, args: &Args) -> ExitCode {
    let total_gremlins = stats.get_total_gremlins();
    let errors = stats.get_errors();

    if errors > 0 {
        eprintln!("\n{}", "Scan completed with errors.".red());
        return ExitCode::FAILURE;
    }

    if total_gremlins > 0 {
        let files = stats.get_files_with_gremlins();
        eprintln!(
            "\n{} found in {} files.",
            format!("{} gremlins", total_gremlins).red().bold(),
            files
        );

        if !args.write {
            eprintln!("Run with {} to fix.", "--write".cyan());
            return ExitCode::FAILURE;
        } else {
            eprintln!("{}", "All gremlins have been fixed.".green().bold());
            return ExitCode::SUCCESS;
        }
    }

    if args.verbose {
        eprintln!("{}", "No gremlins found.".green());
    }

    ExitCode::SUCCESS
}
