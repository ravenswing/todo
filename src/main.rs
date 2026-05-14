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
#[derive(Debug, Copy, Clone)]
enum Status {
    Open,
    Completed,
}

#[derive(Debug, Clone)]
struct Task {
    task: String,
    line: u32,
    status: Status,
    priority: Option<char>,
    projects: Option<Vec<String>>,
}

fn extract_priority(line: &mut str) -> (Option<char>, &str) {
    // Search for (X) at the beginning of the line
    if line.starts_with('(')
        && line.len() >= 3
        && Some(')') == line.chars().nth(2)
        && let Some(c) = line.chars().nth(1)
    {
        // Slice off the "(X)" and trim any following spaces
        let line = line[3..].trim_start();
        (Some(c), line)
    // If not found, return None and unedited line
    } else {
        (None, line)
    }
}

fn extract_projects(line: &mut str) -> (Option<Vec<String>>, &str) {
    // Quick return if no tagged projects are in the line
    if !line.contains("+") {
        return (None, line);
    }

    let mut words: Vec<&str> = line.split_whitespace().collect();
    let mut projects_vec = Vec::new();

    while let Some(word) = words.last() {
        if word.starts_with('+') && word.len() > 1 {
            projects_vec.push(word[1..].to_string()); // Omit the '+' symbol
            words.pop();
        } else {
            break;
        }
    }

    projects_vec.reverse();

    let projects = if projects_vec.is_empty() {
        None
    } else {
        Some(projects_vec)
    };
    let task_body = words.join(" ");
}

fn parse_task(tagged_line: &str, search_str: &str) -> Task {
    // remove the search string
    let line = tagged_line.trim().trim_start_matches(&search_str).trim();

    let (priority, line) = extract_priority(&mut line);
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
