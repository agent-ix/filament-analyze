// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

use std::{env, process::Command};

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(env::var_os("CARGO_MANIFEST_DIR")?)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).ok())
        .flatten()
        .map(|value| value.trim().to_owned())
}

fn watch_git_path(path: &str) {
    if let Some(path) = git_output(&["rev-parse", "--git-path", path]) {
        println!("cargo:rerun-if-changed={path}");
    }
}

fn main() {
    watch_git_path("HEAD");
    watch_git_path("index");
    watch_git_path("packed-refs");
    watch_git_path("refs");
    watch_git_path("refs/tags");

    if let Some(files) = git_output(&["ls-files"]) {
        for file in files.lines() {
            println!("cargo:rerun-if-changed={file}");
        }
    }
    if let Some(refs) = git_output(&["for-each-ref", "--format=%(refname)"]) {
        for reference in refs.lines() {
            watch_git_path(reference);
        }
    }

    let version = git_output(&["describe", "--tags", "--dirty", "--always"])
        .filter(|version| !version.is_empty())
        .map(|version| version.strip_prefix('v').unwrap_or(&version).to_owned())
        .unwrap_or_else(|| env::var("CARGO_PKG_VERSION").expect("Cargo package version"));
    println!("cargo:rustc-env=FILAMENT_BUILD_VERSION={version}");
}
