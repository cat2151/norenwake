use std::fs;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/packed-refs");

    if let Ok(head) = fs::read_to_string(".git/HEAD") {
        if let Some(head_ref) = head.trim().strip_prefix("ref: ") {
            println!("cargo:rerun-if-changed=.git/{head_ref}");
        }
    }

    let git_commit_hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|stdout| stdout.trim().to_string())
        .filter(|hash| !hash.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_COMMIT_HASH={git_commit_hash}");
}
