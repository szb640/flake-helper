//! Integration tests for the `update` module, exercising it through the
//! public library API (`fh::update`).
use fh::update;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const OLD_REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const NEW_REV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

/// The pinned-revision pattern for `flake.nix`, as registered in `PINNED_FILES`.
fn flake_pattern() -> &'static str {
    pattern_for("flake.nix")
}

/// The pinned-revision pattern for `shell.nix`, as registered in `PINNED_FILES`.
fn shell_pattern() -> &'static str {
    pattern_for("shell.nix")
}

fn pattern_for(name: &str) -> &'static str {
    update::PINNED_FILES
        .iter()
        .find(|&&(file, _)| file == name)
        .map(|&(_, pattern)| pattern)
        .expect("known pinned file")
}

static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A uniquely-named temporary directory that is removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let n = DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let thread = std::thread::current()
            .name()
            .map(str::to_owned)
            .unwrap_or_else(|| "t".to_string());
        let path = std::env::temp_dir().join(format!(
            "fh-it-{}-{}-{}",
            std::process::id(),
            thread.replace(':', "_"),
            n
        ));
        fs::create_dir_all(&path).expect("failed to create temp dir");
        TempDir(path)
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent dir");
        }
        fs::write(&path, contents).expect("failed to write temp file");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn updates_flake_nix_pinned_revision() {
    let dir = TempDir::new();
    let path = dir.write("flake.nix", &format!(
        "inputs.nixpkgs.url = \"github:NixOS/nixpkgs/{OLD_REV}\";\n"
    ));

    update::update_file(&path, flake_pattern(), NEW_REV);

    let result = fs::read_to_string(&path).unwrap();
    let expected =
        format!("inputs.nixpkgs.url = \"github:NixOS/nixpkgs/{NEW_REV}\";\n");
    assert_eq!(result, expected);
    assert!(result.contains(NEW_REV));
    assert!(!result.contains(OLD_REV));
}

#[test]
fn preserves_suffix_on_shell_nix() {
    let dir = TempDir::new();
    let path = dir.write("shell.nix", &format!(
        "import (fetchTarball \"https://github.com/NixOS/nixpkgs/archive/{OLD_REV}.tar.gz\");\n"
    ));

    update::update_file(&path, shell_pattern(), NEW_REV);

    let result = fs::read_to_string(&path).unwrap();
    let expected = format!(
        "import (fetchTarball \"https://github.com/NixOS/nixpkgs/archive/{NEW_REV}.tar.gz\");\n"
    );
    assert_eq!(result, expected);
}

#[test]
fn replaces_single_occurrence() {
    let dir = TempDir::new();
    let path = dir.write("flake.nix", &format!(
        "a = \"github:NixOS/nixpkgs/{OLD_REV}\";\nb = \"github:NixOS/nixpkgs/{OLD_REV}\";\n"
    ));

    update::update_file(&path, flake_pattern(), NEW_REV);

    let result = fs::read_to_string(&path).unwrap();
    assert_eq!(result.matches(NEW_REV).count(), 1);
    assert_eq!(result.matches(OLD_REV).count(), 1);
}

#[test]
fn leaves_non_pinning_file_unchanged() {
    let dir = TempDir::new();
    let contents = "inputs.nixpkgs.url = \"github:NixOS/nixpkgs/nixos-unstable\";\n";
    let path = dir.write("flake.nix", contents);

    update::update_file(&path, flake_pattern(), NEW_REV);

    assert_eq!(fs::read_to_string(&path).unwrap(), contents);
}

#[test]
fn leaves_up_to_date_file_unchanged() {
    let dir = TempDir::new();
    let contents = format!("inputs.nixpkgs.url = \"github:NixOS/nixpkgs/{NEW_REV}\";\n");
    let path = dir.write("flake.nix", &contents);

    update::update_file(&path, flake_pattern(), NEW_REV);

    assert_eq!(fs::read_to_string(&path).unwrap(), contents);
}

#[test]
fn missing_file_is_handled_gracefully() {
    let dir = TempDir::new();
    let path = dir.0.join("does-not-exist.nix");

    // Should not panic even though the file is absent.
    update::update_file(&path, flake_pattern(), NEW_REV);
}

#[test]
fn find_files_discovers_nested_files_recursively() {
    let dir = TempDir::new();
    dir.write("flake.nix", "");
    dir.write("shell.nix", "");
    dir.write("sub/deep/flake.nix", "");
    dir.write("sub/shell.nix", "");
    // An unrelated file is ignored by the search.
    dir.write("other.toml", "");
    // A `.nix` file that is not the searched-for name is ignored too.
    dir.write("default.nix", "");

    let found: Vec<PathBuf> = update::find_files(&dir.0, "flake.nix", true).collect();

    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|p| p.ends_with("flake.nix")));
    assert!(found.iter().any(|p| p.ends_with("sub/deep/flake.nix")));
    assert!(found.iter().all(|p| !p.ends_with("shell.nix")));
    assert!(found.iter().all(|p| !p.ends_with("other.toml")));
    assert!(found.iter().all(|p| !p.ends_with("default.nix")));
}

#[test]
fn find_files_direct_only_when_not_recursive() {
    let dir = TempDir::new();
    dir.write("shell.nix", "");
    dir.write("sub/shell.nix", "");

    let found: Vec<PathBuf> = update::find_files(&dir.0, "shell.nix", false).collect();

    assert_eq!(found.len(), 1);
    assert!(found.iter().any(|p| p.ends_with("shell.nix")));
    assert!(found.iter().all(|p| !p.ends_with("sub/shell.nix")));
}

#[test]
fn pinned_file_regexes_are_valid() {
    for &(name, pattern) in update::PINNED_FILES {
        let re = regex::Regex::new(pattern);
        assert!(re.is_ok(), "{name} has an invalid regex: {pattern}");
    }
}

#[test]
fn pinned_file_regexes_capture_prefix_and_revision() {
    let flake = regex::Regex::new(flake_pattern()).unwrap();
    let flake_input = format!("github:NixOS/nixpkgs/{OLD_REV}");
    let flake_caps = flake.captures(&flake_input).unwrap();
    assert_eq!(&flake_caps[1], "github:NixOS/nixpkgs/");
    assert_eq!(&flake_caps[2], OLD_REV);

    let shell = regex::Regex::new(shell_pattern()).unwrap();
    let shell_input =
        format!("https://github.com/NixOS/nixpkgs/archive/{OLD_REV}.tar.gz");
    let shell_caps = shell.captures(&shell_input).unwrap();
    assert_eq!(&shell_caps[1], "https://github.com/NixOS/nixpkgs/archive/");
    assert_eq!(&shell_caps[2], OLD_REV);
    assert_eq!(&shell_caps[3], ".tar.gz");
}
