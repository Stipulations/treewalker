use clap::{Arg, Command};
use std::io::{self};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
enum FileTreeError {
    Io(io::Error),
    Walkdir(walkdir::Error),
    InvalidPath,
}

impl From<io::Error> for FileTreeError {
    fn from(err: io::Error) -> FileTreeError {
        FileTreeError::Io(err)
    }
}

impl From<walkdir::Error> for FileTreeError {
    fn from(err: walkdir::Error) -> FileTreeError {
        FileTreeError::Walkdir(err)
    }
}

fn get_dir_entries(path: &Path, ignore_hidden: bool) -> Result<Vec<PathBuf>, FileTreeError> {
    let mut entries: Vec<PathBuf> = vec![];

    for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
        let entry = entry?;

        if ignore_hidden && entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }

        entries.push(entry.path().to_path_buf());
    }

    entries.sort_by(|a, b| {
        let a_is_dir = a.is_dir();
        let b_is_dir = b.is_dir();
        if a_is_dir == b_is_dir {
            a.cmp(b)
        } else if a_is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(entries)
}

fn print_tree(path: &Path, prefix: &str, ignore_hidden: bool) -> Result<(), FileTreeError> {
    if !path.is_dir() {
        return Err(FileTreeError::InvalidPath);
    }

    let entries = get_dir_entries(path, ignore_hidden)?;

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let file_name = entry.file_name().unwrap().to_string_lossy();

        let new_prefix = if is_last {
            format!("{}└── ", prefix)
        } else {
            format!("{}├── ", prefix)
        };

        let continuation_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        if entry.is_dir() {
            println!("{}{}/", new_prefix, file_name);
            print_tree(entry, &continuation_prefix, ignore_hidden)?;
        } else {
            println!("{}{}", new_prefix, file_name);
        }
    }

    Ok(())
}

fn main() {
    let matches = Command::new("Treewalker")
        .arg(
            Arg::new("path")
                .help("The path to the directory")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("ignore-hidden")
                .long("ignore-hidden")
                .help("Ignore files and folders that start with a '.'")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let path = matches.get_one::<String>("path").expect("Path is required");

    let ignore_hidden = matches.get_flag("ignore-hidden");

    match print_tree(Path::new(path), "", ignore_hidden) {
        Ok(_) => {}
        Err(FileTreeError::Io(err)) => eprintln!("Error reading the directory: {}", err),
        Err(FileTreeError::Walkdir(err)) => eprintln!("Error walking the directory: {}", err),
        Err(FileTreeError::InvalidPath) => eprintln!("Invalid directory path: {}", path),
    }
}