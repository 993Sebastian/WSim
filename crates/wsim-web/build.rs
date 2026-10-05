//! Builds the game data into the module: every `.yaml` file below `data/`, like
//! `wsim_data::load_dir` reads them.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn collect(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("data directory readable")
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            println!("cargo:rerun-if-changed={}", path.display());
            collect(root, &path, out);
        } else if path.extension().is_some_and(|e| e == "yaml") {
            let relative = path.strip_prefix(root).expect("below data");
            let parts: Vec<String> = relative
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            out.push(parts.join("/"));
        }
    }
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let root = manifest
        .join("../../data")
        .canonicalize()
        .expect("data directory");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    collect(&root, &root, &mut files);
    let mut code = String::from(
        "/// The game data: path below `data/` and content of every file.\n\
         pub static DATEN: &[(&str, &str)] = &[\n",
    );
    for relative in &files {
        let absolute = root.join(relative);
        println!("cargo:rerun-if-changed={}", absolute.display());
        code.push_str(&format!(
            "    ({relative:?}, include_str!({:?})),\n",
            absolute.display().to_string()
        ));
    }
    code.push_str("];\n");
    let out = PathBuf::from(env::var("OUT_DIR").expect("set by cargo")).join("daten.rs");
    fs::write(out, code).expect("generated file written");
}
