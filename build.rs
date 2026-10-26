//! Lists the files under src/assets, so the app knows which pictures exist without asking the server.

use std::fs;
use std::path::Path;

fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() {
            walk(&path, root, out);
        } else if let Ok(rel) = path.strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn main() {
    let root = Path::new("src/assets");
    println!("cargo:rerun-if-changed=src/assets");
    let mut files = Vec::new();
    walk(root, root, &mut files);
    files.sort();
    let list: Vec<String> = files.iter().map(|f| format!("    {f:?},")).collect();
    let code = format!("pub const FILES: &[&str] = &[\n{}\n];\n", list.join("\n"));
    let out = Path::new(&std::env::var("OUT_DIR").unwrap()).join("assets.rs");
    fs::write(out, code).unwrap();
}
