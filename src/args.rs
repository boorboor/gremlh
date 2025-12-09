use clap::Parser;
use std::{
    io::IsTerminal,
    path::{Path, PathBuf},
};

pub const CHANNEL_CAPACITY: usize = 128;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,

    #[arg(short, long)]
    pub write: bool,

    #[arg(short, long)]
    pub verbose: bool,

    #[arg(long)]
    pub no_ignore: bool,

    #[arg(long)]
    pub hidden: bool,

    #[arg(long, short = 'j')]
    pub threads: Option<usize>,

    #[clap(skip)]
    pub is_stdin: bool,
}

impl Args {
    pub fn parse_and_finalize() -> Self {
        let mut args = Self::parse();

        if args.paths.len() == 1
            && args.paths[0] == Path::new(".")
            && !std::io::stdin().is_terminal()
        {
            args.is_stdin = true;
        }

        args
    }
}
