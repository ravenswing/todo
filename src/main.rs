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
    line_no: u32,
    status: Status,
    priority: Option<char>,
    projects: Option<Vec<String>>,
}

fn extract_priority(line: &str) -> (Option<char>, &str) {
    // Search for (X) at the beginning of the line
    if line.starts_with('(')
        && line.len() >= 3
        && line.chars().nth(2) == Some(')')
        && let Some(c) = line.chars().nth(1)
    {
        // Slice off the "(X)" and trim any following spaces
        let line = line[3..].trim_start();
        // Return the priority character and the trimmed line
        (Some(c), line)
    } else {
        // If not found, return None and unedited line
        (None, line)
    }
}

fn extract_projects(line: &str) -> (Option<Vec<String>>, String) {
    // Quick return if no tagged projects are in the line
    if !line.contains("+") {
        return (None, line.to_string());
    }

    // Split all the words and a Vec to hold potential project tags
    let mut words: Vec<&str> = line.split_whitespace().collect();
    let mut projects_vec = Vec::new();

    // Starting from the back, search the words for '+' tag
    while let Some(word) = words.last() {
        if word.starts_with('+') && word.len() > 1 {
            // Add to the projects list
            projects_vec.push(word[1..].to_string()); // Omit the '+' symbol
            words.pop();
        } else {
            break;
        }
    }

    // Empty vector for whatever reason just returns None and the string
    if projects_vec.is_empty() {
        (None, words.join(" "))
    } else {
        // Maintain the projects in the order they appear in the line
        projects_vec.reverse();
        (Some(projects_vec), words.join(" "))
    }
}

fn parse_task(tagged_line: &str, search_str: &str, line_no: u32) -> Task {
    // remove the search string
    let line = tagged_line.trim().trim_start_matches(search_str).trim();
    // Split and read the priority from the start of the line
    let (priority, line) = extract_priority(line);
    // Split and read the projects tags from the end of the line
    let (projects, task) = extract_projects(line);
    // Build and return the open task
    Task {
        task,
        line_no,
        status: Status::Open,
        priority,
        projects,
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
                .with_context(|| format!("Failed to open file: '{}'", &path.display()))?;
            let reader = BufReader::new(file);

            let mut first_find = true;
            for (i, line) in reader.lines().enumerate() {
                match line {
                    Ok(tagged_line) => {
                        if tagged_line.contains(&search_str) {
                            if first_find {
                                println!();
                                first_find = false
                            }

                            let task = parse_task(
                                &tagged_line,
                                &search_str,
                                (i + 1).try_into().unwrap_or(0),
                            );

                            println!(
                                "{} : Line {:3} - {}",
                                &path.display(),
                                &task.line_no,
                                &task.task
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
