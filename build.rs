use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=ui");
    let mut hasher = DefaultHasher::new();
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    collect(Path::new("ui/dist"), &mut files);
    files.sort();
    for (name, bytes) in &files {
        name.hash(&mut hasher);
        bytes.hash(&mut hasher);
    }
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("ui_fingerprint.txt");
    std::fs::write(out, format!("{:016x}", hasher.finish())).unwrap();
    tauri_build::build()
}

fn collect(dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if let Ok(bytes) = std::fs::read(&path) {
            out.push((path.to_string_lossy().into_owned(), bytes));
        }
    }
}
