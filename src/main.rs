use std::{
    fs, io,
    io::prelude::*,
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    dir: PathBuf,
    #[arg(short, long)]
    depth: Option<u32>,
    #[arg(short = 'a', long)]
    show_hidden: bool,
}

#[derive(Debug, Default)]
struct Summary {
    dirs: u32,
    files: u32,
}

fn visit_dirs(
    dir: &Path,
    handle: &mut dyn Write,
    prefixes: &mut Vec<&'static str>,
    depth: u32,
    cli: &Cli,
) -> io::Result<Summary> {
    let Cli {
        show_hidden,
        depth: max_depth,
        ..
    } = *cli;
    let mut summary = Summary::default();
    if max_depth.is_none_or(|max| depth < max) {
        match fs::read_dir(dir) {
            Ok(entries) => {
                let mut v: Box<[_]> = entries
                    .filter_map(|r| r.inspect_err(|e| eprintln!("{}", e)).ok())
                    .filter(|e| {
                        show_hidden || e.file_name().to_str().is_none_or(|n| !n.starts_with('.'))
                    })
                    .collect();
                v.sort_by_key(|e| e.file_name());
                for (i, entry) in v.iter().enumerate() {
                    let (prefix, connector) = if i + 1 == v.len() {
                        ("   ", "\u{2514}\u{2500} ")
                    } else {
                        ("\u{2502}  ", "\u{251C}\u{2500} ")
                    };
                    let path = entry.path();
                    let file_name = entry.file_name();
                    writeln!(
                        handle,
                        "{}{}{}",
                        prefixes.concat(),
                        connector,
                        file_name.display(),
                    )?;
                    if path.is_dir() {
                        summary.dirs += 1;
                        prefixes.push(prefix);
                        let Summary { dirs, files } =
                            visit_dirs(&path, handle, prefixes, depth + 1, cli)?;
                        summary.dirs += dirs;
                        summary.files += files;
                        prefixes.pop();
                    } else if path.is_file() {
                        summary.files += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("{}: {}", dir.display(), e);
            }
        }
    }
    Ok(summary)
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let Cli { dir, .. } = &cli;
    let mut handle = io::stdout().lock();
    writeln!(handle, "{}", dir.display())?;
    let Summary { dirs, files } = visit_dirs(dir, &mut handle, &mut vec![], 0, &cli)?;
    writeln!(
        handle,
        "\n{} {}, {} {}",
        dirs,
        if dirs == 1 {
            "directory"
        } else {
            "directories"
        },
        files,
        if files == 1 { "file" } else { "files" }
    )?;
    Ok(())
}
