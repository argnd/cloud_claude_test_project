//! Embeds every story script in assets/story/ into the executable.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let dir = Path::new("assets/story");
    println!("cargo:rerun-if-changed=assets/story");
    let mut files: Vec<_> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "story"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    let mut out = String::from("pub static STORY_FILES: &[(&str, &str)] = &[\n");
    for path in &files {
        println!("cargo:rerun-if-changed={}", path.display());
        let name = path.file_name().unwrap().to_string_lossy();
        out += &format!(
            "    ({name:?}, include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/story/{name}\"))),\n"
        );
    }
    out += "];\n";
    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("story_files.rs");
    fs::write(dest, out).expect("write story_files.rs");
}
