use anyhow::{Context, Result};
use clap::Parser;
use std::io::{BufRead, BufReader};
use std::{fs::File, path::Path};
use walkdir::{DirEntry, WalkDir};

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

fn dir_filter(entry: &DirEntry, ext: &str) -> bool {
    let path = entry.path();
    if let Some(file_ext) = path.extension() {
        println!("{:?}", &file_ext);
        file_ext.to_string_lossy() == ext
    } else {
        false
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

    for entry in WalkDir::new(dir)
        .into_iter()
        //.filter_entry(|e| dir_filter(e, ext))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file()
            && let Some(file_ext) = path.extension()
            && file_ext.to_string_lossy() == ext
        {
            println!("Found file: {}", &path.display());
            let file = File::open(path)
                .with_context(|| format!("Failed to open file: '{:?}'", &path.display()))?;
            let reader = BufReader::new(file);
            for (i, line) in reader.lines().enumerate() {
                match line {
                    Ok(str) => {
                        if str.contains(&search_str) {
                            println!(
                                "{} : Line {} - {}",
                                &path.display(),
                                i,
                                &str.trim().trim_start_matches(&search_str)
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
