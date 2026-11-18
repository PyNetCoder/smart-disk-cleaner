use std::{
    env,
    ffi::OsStr,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process,
};

const CLEANUP_DIRS: [&str; 4] = ["node_modules", "target", ".venv", "build"];

struct Candidate {
    path: PathBuf,
    size: u64,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let Some(root) = parse_root()? else {
        return Ok(());
    };

    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("'{}' is not a directory", root.display()),
        ));
    }

    println!("Scanning {}...", root.display());
    let candidates = find_candidates(&root)?;

    if candidates.is_empty() {
        println!("No cleanup directories found.");
        return Ok(());
    }

    let total_size = candidates
        .iter()
        .fold(0_u64, |total, candidate| total.saturating_add(candidate.size));

    println!("\nCleanup directories:");
    for candidate in &candidates {
        println!("  {:>10}  {}", format_size(candidate.size), candidate.path.display());
    }
    println!("\n{} director{} found, using approximately {}.",
        candidates.len(),
        if candidates.len() == 1 { "y" } else { "ies" },
        format_size(total_size),
    );

    print!("Delete these directories? [y/N]: ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    if !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
        println!("No directories were deleted.");
        return Ok(());
    }

    let mut deleted = 0;
    let mut failed = 0;
    for candidate in &candidates {
        match fs::remove_dir_all(&candidate.path) {
            Ok(()) => {
                println!("Deleted: {}", candidate.path.display());
                deleted += 1;
            }
            Err(error) => {
                eprintln!("Could not delete '{}': {error}", candidate.path.display());
                failed += 1;
            }
        }
    }

    println!("Finished: {deleted} deleted, {failed} failed.");
    if failed > 0 {
        return Err(io::Error::other(format!("failed to delete {failed} director(y/ies)")));
    }

    Ok(())
}

fn parse_root() -> io::Result<Option<PathBuf>> {
    let mut args = env::args_os().skip(1);
    let Some(first) = args.next() else {
        return Ok(Some(env::current_dir()?));
    };

    if first == OsStr::new("--help") || first == OsStr::new("-h") {
        print_help();
        return Ok(None);
    }

    if args.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected at most one directory argument; use --help for usage",
        ));
    }

    Ok(Some(PathBuf::from(first)))
}

fn print_help() {
    println!(
        "Smart Disk Cleaner\n\nUsage:\n  smart-disk-cleaner [DIRECTORY]\n\nScans DIRECTORY recursively for node_modules, target, .venv, and build directories.\nIf DIRECTORY is omitted, the current directory is scanned. Deletion requires confirmation."
    );
}

fn find_candidates(root: &Path) -> io::Result<Vec<Candidate>> {
    let mut candidates = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;

            // Do not follow symbolic links while scanning.
            if !file_type.is_dir() {
                continue;
            }

            let path = entry.path();
            if is_cleanup_directory(&entry.file_name()) {
                candidates.push(Candidate {
                    size: directory_size(&path)?,
                    path,
                });
            } else {
                pending.push(path);
            }
        }
    }

    candidates.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(candidates)
}

fn is_cleanup_directory(name: &OsStr) -> bool {
    CLEANUP_DIRS.iter().any(|candidate| name == OsStr::new(candidate))
}

fn directory_size(root: &Path) -> io::Result<u64> {
    let mut total = 0_u64;
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() {
                total = total.saturating_add(entry.metadata()?.len());
            }
        }
    }

    Ok(total)
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    format!("{value:.1} {}", UNITS[unit])
}
