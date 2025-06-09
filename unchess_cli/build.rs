use std::env::var;
use std::fmt::Write as _;
use std::process::Command;

fn main() {
    let mut ver = String::new();
    ver.push_str(&var("CARGO_PKG_VERSION").unwrap());
    if var("PROFILE").unwrap() != "release" {
        let output = Command::new("git").args(["rev-parse", "HEAD"]).output().unwrap();
        let git_hash = String::from_utf8(output.stdout).unwrap();
        write!(ver, "_{:.8}", git_hash).unwrap();
    }

    println!("cargo:rustc-env=UNCHESS_FULL_VERSION={ver}");
}
