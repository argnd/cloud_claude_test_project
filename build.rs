//! Embeds every story script in assets/story/ into the executable and, on
//! Windows, gives the exe an icon, version information and a manifest.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    story_files();
    windows_resources();
}

fn story_files() {
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

/// Unsigned executables with no version information, icon or manifest look
/// suspicious to antivirus heuristics; this gives ours all three.
fn windows_resources() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    println!("cargo:rerun-if-changed=assets/windows");
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/windows/emberdeep.ico")
        .set_manifest_file("assets/windows/emberdeep.manifest")
        .set("ProductName", "Emberdeep - The Last Lantern")
        .set("FileDescription", "Emberdeep - The Last Lantern")
        .set("CompanyName", "Emberdeep team")
        .set("LegalCopyright", "Copyright (c) 2026 Emberdeep team")
        .set("OriginalFilename", "emberdeep.exe")
        .set("InternalName", "emberdeep");
    // Needs rc.exe (Windows SDK) or windres; without them the game still
    // builds, just without the metadata.
    if let Err(e) = res.compile() {
        println!("cargo:warning=Windows resources skipped: {e}");
    }
}
