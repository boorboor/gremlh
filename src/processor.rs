use crate::args::{Args, CHANNEL_CAPACITY};
use crate::scanner::{scan_line, GremlinLoc};
use crate::stats::ScanStats;
use anyhow::{Context, Result};
use colored::*;
use ignore::{WalkBuilder, WalkState};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc};
use std::thread;
use tempfile::NamedTempFile;

#[derive(Debug)]
enum ScanEvent {
    Clean {
        path: PathBuf,
    },
    GremlinsFound {
        path: PathBuf,
        gremlins: Vec<GremlinLoc>,
    },
    SkippedBinary {
        path: PathBuf,
    },
    Error {
        path: PathBuf,
        message: String,
    },
    Warning {
        path: PathBuf,
        message: String,
    },
}

enum OutputDest {
    File(BufWriter<NamedTempFile>),
    None,
}

pub fn run(args: &Args) -> Result<Arc<ScanStats>> {
    if args.verbose && !args.is_stdin {
        eprintln!("Scanning {} path(s)...", args.paths.len());
    }

    if args.is_stdin {
        return process_stdin_and_report(args);
    }

    // Validate all paths exist before starting
    for path in &args.paths {
        if !path.exists() && path != Path::new(".") {
            anyhow::bail!("Path does not exist: {}", path.display());
        }
    }

    scan_path_parallel(args)
}

fn process_stdin_and_report(args: &Args) -> Result<Arc<ScanStats>> {
    let stats = Arc::new(ScanStats::default());

    if args.verbose {
        eprintln!("{}", "Reading from STDIN...".yellow());
    }

    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    let mut gremlins = Vec::new();

    for (idx, line_res) in reader.lines().enumerate() {
        let line = line_res?;
        let (fixed_line, line_gremlins) = scan_line(&line, idx + 1);

        if !line_gremlins.is_empty() {
            gremlins.extend(line_gremlins);
        }
        writeln!(stdout_lock, "{}", fixed_line)?;
    }

    if !gremlins.is_empty() {
        stats.add_gremlins(gremlins.len());

        let stderr = io::stderr();
        let mut lock = stderr.lock();
        report_gremlins_minimal(&mut lock, Path::new("<stdin>"), &gremlins)?;
    }
    Ok(stats)
}

fn scan_path_parallel(args: &Args) -> Result<Arc<ScanStats>> {
    let stats = Arc::new(ScanStats::default());
    let (tx, rx) = mpsc::sync_channel(CHANNEL_CAPACITY);

    let printer_handle = {
        let verbose = args.verbose;
        thread::spawn(move || {
            handle_printing(rx, verbose);
        })
    };

    if args.paths.is_empty() {
        anyhow::bail!("No paths provided to scan");
    }

    // Initialize builder with the first path
    let mut builder = WalkBuilder::new(&args.paths[0]);

    // Add any additional paths to the walker
    for path in args.paths.iter().skip(1) {
        builder.add(path);
    }

    builder.git_ignore(!args.no_ignore).hidden(!args.hidden);

    if let Some(threads) = args.threads {
        builder.threads(threads);
    }

    let walker = builder.build_parallel();

    walker.run(|| {
        let stats = Arc::clone(&stats);
        let args = args.clone();
        let tx = tx.clone();

        Box::new(move |result| {
            let entry = match result {
                Ok(entry) => entry,
                Err(err) => {
                    stats.inc_errors();
                    let _ = tx.send(ScanEvent::Error {
                        path: PathBuf::from("?"),
                        message: err.to_string(),
                    });
                    return WalkState::Continue;
                }
            };

            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                return WalkState::Continue;
            }

            let path = entry.path().to_path_buf();

            match process_file(&path, &args, &tx) {
                Ok(event) => {
                    stats.inc_total_files();
                    if let ScanEvent::GremlinsFound { gremlins, .. } = &event {
                        stats.add_gremlins(gremlins.len());
                    }
                    let _ = tx.send(event);
                }
                Err(e) => {
                    stats.inc_errors();
                    let _ = tx.send(ScanEvent::Error {
                        path,
                        message: e.to_string(),
                    });
                }
            }
            WalkState::Continue
        })
    });

    drop(tx);
    if let Err(e) = printer_handle.join() {
        eprintln!("Error waiting for printer thread: {:?}", e);
    }

    Ok(stats)
}

fn process_file(path: &Path, args: &Args, tx: &mpsc::SyncSender<ScanEvent>) -> Result<ScanEvent> {
    let mut file = File::open(path)?;

    // Binary Check
    let mut head = [0u8; 4096];
    let n = file.read(&mut head)?;
    if is_buffer_binary(&head[..n]) {
        return Ok(ScanEvent::SkippedBinary {
            path: path.to_path_buf(),
        });
    }
    file.seek(SeekFrom::Start(0))?;

    let reader = BufReader::new(file);

    // Configure Output Destination
    let mut writer = if args.write {
        let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));
        let temp_file = tempfile::Builder::new()
            .prefix(".gremlin_tmp_")
            .tempfile_in(parent_dir)?;

        OutputDest::File(BufWriter::new(temp_file))
    } else {
        OutputDest::None
    };

    let mut file_gremlins = Vec::new();

    for (line_idx, line_result) in reader.lines().enumerate() {
        let line = line_result.context("Failed to read line")?;
        let (fixed_line, gremlins) = scan_line(&line, line_idx + 1);

        if !gremlins.is_empty() {
            file_gremlins.extend(gremlins);
        }

        match &mut writer {
            OutputDest::File(w) => {
                w.write_all(fixed_line.as_bytes())?;
                w.write_all(b"\n")?;
            }
            OutputDest::None => {}
        }
    }

    // Finalize / Flush
    match writer {
        OutputDest::File(mut w) => {
            if !file_gremlins.is_empty() {
                w.flush()?;
                let temp_file = w.into_inner().map_err(|e| e.into_error())?;

                if let Ok(metadata) = fs::metadata(path) {
                    if let Err(e) = temp_file.as_file().set_permissions(metadata.permissions()) {
                        let _ = tx.send(ScanEvent::Warning {
                            path: path.to_path_buf(),
                            message: format!("Could not preserve file permissions: {}", e),
                        });
                    }
                }

                temp_file
                    .persist(path)
                    .context("Failed to persist changes to file")?;
            }
        }
        OutputDest::None => {}
    }

    if file_gremlins.is_empty() {
        return Ok(ScanEvent::Clean {
            path: path.to_path_buf(),
        });
    }

    Ok(ScanEvent::GremlinsFound {
        path: path.to_path_buf(),
        gremlins: file_gremlins,
    })
}

fn handle_printing(rx: mpsc::Receiver<ScanEvent>, verbose: bool) {
    let stderr = io::stderr();
    let mut lock = stderr.lock();

    for event in rx {
        match event {
            ScanEvent::Clean { path } => {
                if verbose {
                    let _ = writeln!(lock, "{} {}", "clean:".dimmed(), path.display());
                }
            }
            ScanEvent::SkippedBinary { path } => {
                if verbose {
                    let _ = writeln!(lock, "{} {}", "binary:".dimmed(), path.display());
                }
            }
            ScanEvent::Error { path, message } => {
                let _ = writeln!(
                    lock,
                    "{}: {}: {}",
                    "error".red().bold(),
                    path.display(),
                    message
                );
            }
            ScanEvent::Warning { path, message } => {
                if verbose {
                    let _ = writeln!(
                        lock,
                        "{}: {}: {}",
                        "warning".yellow().bold(),
                        path.display(),
                        message
                    );
                }
            }
            ScanEvent::GremlinsFound { path, gremlins } => {
                let _ = report_gremlins_minimal(&mut lock, &path, &gremlins);
            }
        }
    }
}

fn report_gremlins_minimal(
    writer: &mut impl Write,
    path: &Path,
    gremlins: &[GremlinLoc],
) -> Result<()> {
    for g in gremlins {
        let loc = format!("{}:{}:{}", path.display(), g.line, g.col);
        let char_display = g.escape_char().magenta();

        let desc = if g.description.contains("Security") {
            g.description.red().bold()
        } else {
            g.description.yellow()
        };

        writeln!(writer, "{}: found {} ({})", loc.bold(), char_display, desc)?;
    }
    Ok(())
}

fn is_buffer_binary(buffer: &[u8]) -> bool {
    buffer.contains(&0)
}
