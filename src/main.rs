use clap::Parser;

use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    dir: PathBuf,
    #[arg(short = 'a', long)]
    show_hidden: bool,
    #[arg(short, long)]
    depth: Option<usize>,
}

fn visit_dirs(
    dir: &Path,
    lock: &mut impl Write,
    paddings: &mut String,
    depth: Option<usize>,
    cli: &Cli,
) -> io::Result<()> {
    if dir.is_dir() && depth.is_none_or(|d| d > 0) {
        let mut entries = fs::read_dir(dir)?
            .filter_map(|res| {
                res.inspect_err(|e| eprintln!("{}: {}", dir.display(), e))
                    .ok()
            })
            .filter(|e| cli.show_hidden || !e.file_name().as_encoded_bytes().starts_with(b"."))
            .peekable();
        while let Some(entry) = entries.next() {
            let (padding, prefix) = if entries.peek().is_some() {
                ("|   ", "|-- ")
            } else {
                ("    ", "`-- ")
            };
            let file_name = entry.file_name();
            writeln!(
                lock,
                "{}{}{}",
                paddings,
                prefix,
                file_name.display()
            )?;

            let path = entry.path();
            if path.is_dir() {
                paddings.push_str(padding);
                if let Err(e) = visit_dirs(&path, lock, paddings, depth.map(|d| d - 1), cli) {
                    eprintln!("{}: {}", path.display(), e);
                }
                paddings.truncate(paddings.len() - padding.len());
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let mut lock = io::stdout().lock();
    writeln!(lock, "{}", cli.dir.display())?;
    visit_dirs(&cli.dir, &mut lock, &mut String::new(), cli.depth, &cli)?;

    Ok(())
}
