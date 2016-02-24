#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate docopt;
extern crate rustc_serialize;
extern crate walkdir;

use std::ffi::OsString;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};
use docopt::Docopt;
use walkdir::{DirEntry, WalkDirIterator};

const USAGE: &'static str = "
Run cargo command on multiple crates

Usage:
    cargo multi (-v | --version)
    cargo multi (-h | --help)
    cargo multi [options] [--] <cmd>...

Options:
    -v --version    Show version.
    -h --help       Show this help.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_version: bool,
    cmd_multi: String,
    arg_cmd: Vec<String>,
}

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

fn display_path(path: &PathBuf) {
    path.file_name()
        .and_then(|p| p.to_str())
        .map(|p| println!("{}:", p));
}

fn report_output(output: Output) {
    if output.status.success() {
        print_ident(output.stdout);
    } else {
        print_ident(output.stderr);
    }
    // match output.status.success() {
    //     true => print_ident(output.stdout),
    //     false => print_ident(output.stderr),
    // }
    println!("");
}

const CARGO: &'static str = "cargo";
const MIN_DEPTH: usize = 1;
const MAX_DEPTH: usize = 1;

fn main() {
    let version = Some(env!("CARGO_PKG_VERSION").to_owned());
    let setup_parser = |d: Docopt| d.version(version).options_first(true);

    let args: Args = Docopt::new(USAGE)
                         .map(setup_parser)
                         .and_then(|d| d.decode())
                         //.unwrap();
                         .unwrap_or_else(|e| e.exit());

    println!("{:?}", args);

    let mut cmd = Command::new(CARGO);
    let mut banner = String::from("Executing ") + CARGO;

    for arg in args.arg_cmd {
        cmd.arg(OsString::from(&arg));
        banner = banner + " " + &arg;
    }

    announce(&banner);

    let is_crate = |e: &DirEntry| e.path().join("Cargo.toml").exists();
    let to_path_buf = |e: DirEntry| e.path().to_path_buf();
    let execute = move |p| cmd.current_dir(p).output().map(report_output);

    let cwd = env::current_dir().unwrap();
    walkdir::WalkDir::new(cwd)
        .min_depth(MIN_DEPTH)
        .max_depth(MAX_DEPTH)
        .into_iter()
        .filter_entry(is_crate)
        .filter_map(|e| e.ok())
        .map(to_path_buf)
        .inspect(display_path)
        .map(execute)
        .last(); // XXX Non-idiomatic perhaps, but gets the job done. May need to revisit later.
}
