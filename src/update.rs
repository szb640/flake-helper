/// Logic for the `update` action.
use std::path::{Path, PathBuf};
use log::{debug, info, warn, error};

/// Run the `update` action.
///
/// If `recurse` is false, process a `flake.nix` in the current directory.
/// If `recurse` is true, recursively find every `flake.nix` under the current
/// directory and process each one.
pub fn run(recurse: bool) {
    let nixos_version = get_current_nixos_revision();
    debug!("Current NixOS revision: {nixos_version}");
    
    if recurse {
        let cwd = std::env::current_dir().expect("failed to get current directory");
        for flake in find_flakes_recursive(&cwd) {
            update_flake(&flake, &nixos_version);
        }
    } else {
        let flake = std::env::current_dir()
            .expect("failed to get current directory")
            .join("flake.nix");
        if flake.exists() {
            update_flake(&flake, &nixos_version);
        } else {
            warn!("no flake.nix found at {}", flake.display());
        }
    }
}

/// Process a single flake file.
///
/// The caller is responsible for ensuring the file exists.
fn update_flake(flake: &Path, nixos_version: &str) {
    debug!("Starting update of {}...", flake.display());
    let contents = match std::fs::read_to_string(flake) {
        Ok(contents) => contents,
        Err(err) => {
            error!("error reading {}: {err}", flake.display());
            return;
        }
    };

    let re = regex::Regex::new(r"github:NixOS/nixpkgs/([0-9a-f]{40})")
        .expect("invalid nixpkgs pinned regex");

    if !re.is_match(&contents) {
        error!(
            "warning: {} does not pin nixpkgs to a revision; skipping",
            flake.display()
        );
        return;
    }

    // Replace every matched revision part with the current system revision.
    let updated =
        re.replace_all(&contents, format!("github:NixOS/nixpkgs/{nixos_version}"));

    // Only report the file if the content actually changed.
    if updated == contents {
        info!("Flake already up-to-date: {}", flake.display());
        return;
    }

    if let Err(err) = std::fs::write(flake, updated.as_bytes()) {
        error!("error writing {}: {err}", flake.display());
        return;
    }

    info!("Updated flake: {}", flake.display());
}

/// Get the current system's NixOS revision via `nixos-version --revision`.
fn get_current_nixos_revision() -> String {
    let output = std::process::Command::new("nixos-version")
        .arg("--revision")
        .output()
        .expect("failed to run nixos-version");
    String::from_utf8(output.stdout)
        .expect("nixos-version output was not valid UTF-8")
        .trim()
        .to_string()
}

/// Find every `flake.nix` under `root`.
fn find_flakes_recursive(root: &Path) -> impl Iterator<Item = PathBuf> {
    let matcher = globmatch::Builder::new("**/flake.nix")
        .build(root)
        .expect("failed to build flake.nix glob");
    matcher.into_iter().filter_map(|item| match item {
        Ok(path) => Some(path),
        Err(err) => {
            warn!("warning: {err}");
            None
        }
    })
}
