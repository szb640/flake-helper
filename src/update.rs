/// Logic for the `update` action.
use std::path::{Path, PathBuf};
use log::{debug, info, warn, error};

/// Run the `update` action.
///
/// If `recurse` is false, process a `flake.nix` and `shell.nix` in the current
/// directory.
/// If `recurse` is true, recursively find every `flake.nix` and `shell.nix`
/// under the current directory and process each one.
pub fn run(recurse: bool) {
    let nixos_version = get_current_nixos_revision();
    debug!("Current NixOS revision: {nixos_version}");

    // Resolve the working directory once and loop over every pinned file,
    // updating whichever are found with their matching pin pattern.
    let cwd = std::env::current_dir().expect("failed to get current directory");
    let mut found = false;
    for (name, pattern) in PINNED_FILES {
        for file in find_files(&cwd, name, recurse) {
            found = true;
            update_file(&file, pattern, &nixos_version);
        }
        
    }
    if !found {
        warn!("no files found under {}", cwd.display());
    }
}

/// The different ways nixpkgs may be pinned to a revision, keyed by file name.
///
/// * `flake.nix` pins via a flake input URL: `github:NixOS/nixpkgs/<rev>`.
/// * `shell.nix` pins via a tarball URL: `.../nixpkgs/archive/<rev>.tar.gz`.
///
/// Every pattern captures the prefix as group 1 and the revision as group 2,
/// so a single replacement routine can rewrite any of them.
///
/// The keys are also used to discover the files (both directly in the current
/// directory and under recursion), so adding a new file type here teaches `fh`
/// how to find and update it.
pub const PINNED_FILES: &[(&str, &str)] = &[
    ("flake.nix", "(github:NixOS/nixpkgs/)([0-9a-f]{40})"),
    (
        "shell.nix",
        "(https://github.com/NixOS/nixpkgs/archive/)([0-9a-f]{40})(\\.tar\\.gz)",
    ),
];

/// Process a single nix file.
///
/// `pattern` is the pinned-revision regex for this file, taken from
/// `PINNED_FILES`. The caller is responsible for ensuring the file exists.
pub fn update_file(path: &Path, pattern: &str, nixos_version: &str) {
    debug!("Starting update of {}...", path.display());
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            error!("error reading {}: {err}", path.display());
            return;
        }
    };

    let re = regex::Regex::new(pattern).expect("invalid nixpkgs pinned regex");

    if !re.is_match(&contents) {
        error!(
            "warning: {} does not pin nixpkgs to a revision; skipping",
            path.display()
        );
        return;
    }

    let updated = re.replace(&contents, |caps: &regex::Captures| {
        let full = &caps[0];
        let prefix = &caps[1];
        let rev = &caps[2];
        // Preserve any suffix after the revision (e.g. `.tar.gz`).
        let suffix = &full[prefix.len() + rev.len()..];
        format!("{prefix}{nixos_version}{suffix}")
    });

    if updated == contents {
        info!("File already up-to-date: {}", path.display());
        return;
    }

    if let Err(err) = std::fs::write(path, updated.as_bytes()) {
        error!("error writing {}: {err}", path.display());
        return;
    }

    info!("Updated file: {}", path.display());
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

/// Find every file named `name` under `root`.
///
/// If `recursive` is true, descend into subdirectories; otherwise only search
/// `root` directly. Returns the paths found, in no particular order.
pub fn find_files<'a>(
    root: &'a Path,
    name: &str,
    recursive: bool,
) -> impl Iterator<Item = PathBuf> + 'a {
    let glob = if recursive {
        format!("**/{name}")
    } else {
        name.to_owned()
    };
    let matcher = globmatch::Builder::new(&glob)
        .build(root)
        .expect("failed to build nix file glob");
    matcher.into_iter().filter_map(move |item| match item {
        Ok(path) => Some(path),
        Err(err) => {
            warn!("warning: {err}");
            None
        }
    })
}

