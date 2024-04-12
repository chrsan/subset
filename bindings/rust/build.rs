use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir.join("../..");
    let out_dir = env::var("OUT_DIR").unwrap();
    let build_dir = Path::new(&out_dir).join("build");
    if !build_dir.join("build.ninja").exists() {
        let build_type = if env::var("PROFILE").unwrap() == "debug" {
            "debug"
        } else {
            "release"
        };
        let mut cmd = Command::new("meson");
        cmd.current_dir(&root_dir);
        cmd.arg("setup");
        cmd.arg("--buildtype");
        cmd.arg(build_type);
        cmd.arg(&build_dir);
        assert!(cmd.status().unwrap().success());
    }
    let mut cmd = Command::new("meson");
    cmd.current_dir(&build_dir);
    cmd.arg("compile");
    assert!(cmd.status().unwrap().success());
    println!(
        "cargo:rustc-link-search=native={}",
        build_dir.join("extern").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        build_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=harfbuzz");
    println!("cargo:rustc-link-lib=static=sheenbidi");
    println!("cargo:rustc-link-lib=static=subset");
    println!(
        "cargo:rerun-if-changed={}",
        root_dir.join("meson.build").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        root_dir.join("extern").display()
    );
    println!("cargo:rerun-if-changed={}", root_dir.join("lib").display());
}
