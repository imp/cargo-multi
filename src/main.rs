extern crate walkdir;

use std::ffi::OsString;
use std::env;
use std::process::{Command, Output};
use walkdir::WalkDirIterator;

fn announce(banner: &str) {
    let mut line = String::new();
    for _ in 0..banner.len() {
        line.push('-');
    }
    println!("{}", line);
    println!("{}", banner);
    println!("{}", line);
}

fn print_ident(buf: Vec<u8>) {
    for line in String::from_utf8_lossy(&buf[..]).lines() {
        println!("        {}", line);
    }
}

fn report_output(output: Output) {
    if output.status.success() {
        print_ident(output.stdout);
    } else {
        print_ident(output.stderr);
    }
    println!("");
}

const CARGO: &'static str = "cargo";
const MIN_DEPTH: usize = 1;
const MAX_DEPTH: usize = 1;

fn main() {
    let mut cmd = Command::new(CARGO);
    let mut banner = String::from("Executing ") + CARGO;
    for arg in env::args().skip(2) {
        cmd.arg(OsString::from(&arg));
        banner = banner + " " + &arg;
    }

    announce(&banner);

    let cwd = env::current_dir().unwrap();
    walkdir::WalkDir::new(cwd)
        .min_depth(MIN_DEPTH)
        .max_depth(MAX_DEPTH)
        .into_iter()
        .filter_entry(|e| e.path().join("Cargo.toml").exists())
        .map(|e| e.ok().unwrap().path().to_path_buf())
        .map(|p| {
            println!("{}:", p.display());
            cmd.current_dir(p).output().unwrap()
        })
        .map(|o| report_output(o))
        .last();
}
