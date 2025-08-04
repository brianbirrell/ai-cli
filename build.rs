use std::process::Command;

fn main() {
    // Get the git commit hash
    let commit_hash = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Get the git commit hash (short version)
    let commit_hash_short = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Check if there are uncommitted changes
    let dirty = Command::new("git")
        .args(&["diff-index", "--quiet", "HEAD", "--"])
        .output()
        .map(|output| !output.status.success())
        .unwrap_or(false);

    // Get the build timestamp
    let build_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    // Set the build-time constants
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", commit_hash);
    println!("cargo:rustc-env=GIT_COMMIT_HASH_SHORT={}", commit_hash_short);
    println!("cargo:rustc-env=GIT_DIRTY={}", if dirty { "dirty" } else { "clean" });
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);

    // Re-run if git information changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
} 