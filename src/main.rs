// use anyhow::{Context, Result};
use std::path::Path;
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

fn main() {
    println!("Hello, world!");

    let dir = Path::new("/home/revans/personal/todo/data");
    let ext = "py";

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
            println!("Found file: {:?}", &path)
        }
    }
}
