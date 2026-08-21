/// Logic for the `cache` action.
use std::path::{Path, PathBuf};

/// Run the `cache` action.
///
/// If `recurse` is false, process a `flake.nix` in the current directory.
/// If `recurse` is true, recursively find every `flake.nix` under the current
/// directory and process each one.
pub fn run(recurse: bool) {
    if recurse {
        let cwd = std::env::current_dir().expect("failed to get current directory");
        for flake in find_flakes_recursive(&cwd) {
            populate_cache(&flake);
        }
    } else {
        let flake = std::env::current_dir()
            .expect("failed to get current directory")
            .join("flake.nix");
        if flake.exists() {
            populate_cache(&flake);
        } else {
            eprintln!("no flake.nix found at {}", flake.display());
        }
    }
}

/// Dummy implementation: populate the cache for a single flake file.
///
/// The caller is responsible for ensuring the file exists.
fn populate_cache(flake: &Path) {
    // TODO: replace with the real cache-population logic.
    println!("Cached (dummy): {}", flake.display());
}

/// Find every `flake.nix` under `root`.
fn find_flakes_recursive(root: &Path) -> impl Iterator<Item = PathBuf> {
    let matcher = globmatch::Builder::new("**/flake.nix")
        .build(root)
        .expect("failed to build flake.nix glob");
    matcher.into_iter().filter_map(|item| match item {
        Ok(path) => Some(path),
        Err(err) => {
            eprintln!("warning: {err}");
            None
        }
    })
}
