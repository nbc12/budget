use std::path::Path;
use std::process::Command;

/// Builds the Svelte frontend (frontend/) into frontend/dist/ so it can be
/// embedded into the binary via `rust_embed`. Only reruns when frontend
/// source files actually change, thanks to the rerun-if-changed hints below.
fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let frontend_dir = Path::new(manifest_dir).join("..").join("frontend");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", frontend_dir.join("src").display());
    println!("cargo:rerun-if-changed={}", frontend_dir.join("index.html").display());
    println!("cargo:rerun-if-changed={}", frontend_dir.join("package.json").display());
    println!("cargo:rerun-if-changed={}", frontend_dir.join("package-lock.json").display());
    println!("cargo:rerun-if-changed={}", frontend_dir.join("vite.config.ts").display());
    println!("cargo:rerun-if-changed={}", frontend_dir.join("svelte.config.js").display());
    println!("cargo:rerun-if-changed={}", frontend_dir.join("tsconfig.json").display());

    if !frontend_dir.exists() {
        panic!("frontend/ directory not found at {}", frontend_dir.display());
    }

    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };

    if !frontend_dir.join("node_modules").exists() {
        let status = Command::new(npm)
            .arg("install")
            .current_dir(&frontend_dir)
            .status()
            .expect("failed to run `npm install` for frontend/ -- is Node.js/npm installed and on PATH?");
        if !status.success() {
            panic!("`npm install` failed for frontend/");
        }
    }

    let status = Command::new(npm)
        .args(["run", "build"])
        .current_dir(&frontend_dir)
        .status()
        .expect("failed to run `npm run build` for frontend/ -- is Node.js/npm installed and on PATH?");
    if !status.success() {
        panic!("`npm run build` failed for frontend/ (see output above)");
    }
}
