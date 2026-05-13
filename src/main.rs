use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::{fs::File, path::Path};
use walkdir::{DirEntry, WalkDir};

fn dir_filter(entry: &DirEntry, ext: &str) -> bool {
    let path = entry.path();
    if let Some(file_ext) = path.extension() {
        println!("{:?}", &file_ext);
        file_ext.to_string_lossy() == ext
    } else {
        false
    }
}

fn main() -> Result<()> {
    println!("Hello, world!");

    let dir = Path::new("/home/revans/personal/todo/data");
    let ext = "py";
    let search_str = "# TODO!";

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
            println!("Found file: {:?}", &path);
            let file =
                File::open(path).with_context(|| format!("Failed to open file: '{:?}'", &path))?;
            let reader = BufReader::new(file);
            for (i, line) in reader.lines().enumerate() {
                match line {
                    Ok(str) => {
                        if str.contains(search_str) {
                            println!(
                                "{:?}: Line {} - {:?}",
                                &path,
                                i,
                                &str.trim().trim_start_matches(search_str)
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
