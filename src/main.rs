use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::{fs::File, path::Path};
use walkdir::{DirEntry, WalkDir};

// File to store default directories to ignore when searching
const IGNORE_DIRS: &str = include_str!("../.todoignore");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to recursively search in
    #[arg(short, long, default_value = ".")]
    dir: String,
    /// File extension to filter check in (e.g., "rs", "py", "txt")
    #[arg(short, long)]
    ext: String,
    /// Macro-style tag to search for
    #[arg(short, long, default_value = "TODO!")]
    search: String,
}

fn filter_ignored(entry: &DirEntry, ignore: &HashSet<String>) -> bool {
    if entry.file_type().is_dir()
        && let Some(name) = entry.file_name().to_str()
        && ignore.contains(name)
    {
        false
    } else {
        true
    }
}

fn get_comment_str(ext: &str) -> &'static str {
    match ext {
        // C-style comments with //
        "rs" | "c" | "cpp" | "h" | "hpp" | "js" | "ts" | "java" | "go" | "cs" => "//",

        // Python-style comments with #
        "py" | "sh" | "rb" | "yaml" | "yml" | "toml" | "pl" => "#",

        // Lua-style comments with --
        "sql" | "lua" | "hs" => "--",

        // Default = without comment (e.g. markdown/txt)
        _ => "",
    }
}
fn main() -> Result<()> {
    let args = Args::parse();

    println!("todo - Starting compilation...");

    let ext = args.ext.trim_start_matches('.');
    let dir = Path::new(&args.dir);
    let search_str = format!("{} {}", get_comment_str(ext), args.search);

    let ignore_dirs: HashSet<String> = IGNORE_DIRS
        .lines() // -> &str
        .map(|line| line.trim()) // trims whitespace ->  &str
        .filter(|line| !line.is_empty() && !line.starts_with('#')) // keep only valid, non-comment
        .map(|line| line.to_string()) // convert the surviving &str back to owned Strings
        .collect();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| filter_ignored(e, &ignore_dirs))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file()
            && let Some(file_ext) = path.extension()
            && file_ext.to_string_lossy() == ext
        {
            let file = File::open(path)
                .with_context(|| format!("Failed to open file: '{:?}'", &path.display()))?;
            let reader = BufReader::new(file);

            let mut first_find = true;
            for (i, line) in reader.lines().enumerate() {
                match line {
                    Ok(str) => {
                        if str.contains(&search_str) {
                            if first_find {
                                println!();
                                first_find = false
                            }
                            println!(
                                "{} : Line {} - {}",
                                &path.display(),
                                i,
                                &str.trim().trim_start_matches(&search_str).trim()
                            );
                        }
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                }
            }
        }
    }
    Ok(())
}
