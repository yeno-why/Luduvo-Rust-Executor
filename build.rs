use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bin_dir = out_dir.join("bins");
    fs::create_dir_all(&bin_dir).unwrap();

    println!("cargo:rerun-if-changed=asm");
    println!("cargo:rerun-if-changed=tools/nasm.exe");
    println!("cargo:rerun-if-changed=build.rs");

    let nasm = manifest_dir.join("tools").join("nasm.exe");
    if !nasm.exists() {
        panic!("bundled assembler missing: {}\n\
            Download nasm.exe from https://www.nasm.us and place it in tools.",
            nasm.display()
        );
    }

    let asm_dir = manifest_dir.join("asm");
    if !asm_dir.exists() {
        return;
    }

    let mut generated = String::new();

    for entry in fs::read_dir(&asm_dir).unwrap() {
        let path = entry.unwrap().path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext != "asm" && ext != "s" {
            continue;
        }

        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let bin = bin_dir.join(format!("{}.bin", stem));

        let status = Command::new(&nasm)
            .arg("-f")
            .arg("bin")
            .arg(&path)
            .arg("-o")
            .arg(&bin)
            .status()
            .unwrap_or_else(|e| panic!("failed to spawn {}: {e}", nasm.display()));

        if !status.success() {
            panic!("nasm failed for {:?}", path);
        }

        let name = format!("ASM_{}", stem.to_uppercase().replace(|c: char| !c.is_alphanumeric(), "_"));
        generated.push_str(
            &format!("pub static {name}: &[u8] = include_bytes!(r\"{}\");\n",
            bin.display()
        ));
    }

    fs::write(out_dir.join("asm_blobs.rs"), generated).unwrap();
}
