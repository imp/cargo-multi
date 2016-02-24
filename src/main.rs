#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate rustc_serialize;
extern crate walkdir;
extern crate clap;

use std::ffi::OsString;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};
use walkdir::{DirEntry, WalkDirIterator};
use clap::{Arg, App, SubCommand, AppSettings};


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


    let version = env!("CARGO_PKG_VERSION");

    let app_m = App::new("cargo multi")
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("multi")
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::TrailingVarArg)
                .about("Run cargo command on multiple crates")
                .version(version)
                .arg(Arg::from_usage("<cmd>... 'cargo command to run'"))
        )
        .get_matches();

    let arg_cmd = app_m.subcommand_matches("multi").unwrap().values_of("cmd").unwrap();

    let mut cmd = Command::new(CARGO);
    let mut banner = String::from("Executing ") + CARGO;

    for arg in arg_cmd {
        cmd.arg(OsString::from(&arg));
        banner = banner + " " + arg;
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
