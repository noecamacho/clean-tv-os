//! Resolves the build version: `LUME_VERSION_NAME` (leading "v"s stripped), else the short git
//! commit (`-dirty` when a tracked file has uncommitted changes), else `dev`.

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn from_git() -> Option<String> {
    let hash = git(&["rev-parse", "--short", "HEAD"]).filter(|h| !h.is_empty())?;
    for path in ["HEAD", "index"] {
        if let Some(p) = git(&["rev-parse", "--git-path", path]) {
            println!("cargo:rerun-if-changed={p}");
        }
    }
    if let Some(head) = git(&["symbolic-ref", "-q", "HEAD"]) {
        if let Some(p) = git(&["rev-parse", "--git-path", &head]) {
            println!("cargo:rerun-if-changed={p}");
        }
    }
    let dirty =
        git(&["status", "--porcelain", "--untracked-files=no"]).is_some_and(|s| !s.is_empty());
    Some(if dirty { format!("{hash}-dirty") } else { hash })
}

fn main() {
    println!("cargo:rerun-if-env-changed=LUME_VERSION_NAME");
    let env = std::env::var("LUME_VERSION_NAME")
        .map(|v| v.trim().trim_start_matches('v').to_string())
        .ok()
        .filter(|v| !v.is_empty());
    let version = env.or_else(from_git).unwrap_or_else(|| "dev".to_string());
    println!("cargo:rustc-env=LUME_BUILD_VERSION={version}");
}
