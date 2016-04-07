#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate clap;
extern crate itertools;
extern crate walkdir;

use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};
use clap::{App, SubCommand, AppSettings};
use itertools::Itertools;
use walkdir::{DirEntry, WalkDirIterator};


fn announce(banner: &str) {
    let mut line = String::new();
    for _ in 0..banner.len() {
        line.push('-');
    }
    println!("{}", line);
    println!("{}", banner);
    println!("{}", line);
}

fn display_path(path: &PathBuf) {
    path.file_name()
        .and_then(|p| p.to_str())
        .map(|p| println!("{}:", p));
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
    // I am still not sure what is more idiomatic - the 'if' above or the 'match' below
    //
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

    let matches = App::new(CARGO)
                      .bin_name(CARGO)
                      .version(crate_version!())
                      .about("Run cargo command on multiple crates")
                      .setting(AppSettings::SubcommandRequired)
                      .setting(AppSettings::ArgRequiredElseHelp)
                      .subcommand(SubCommand::with_name("multi")
                                      .version(crate_version!())
                                      .setting(AppSettings::ArgRequiredElseHelp)
                                      .setting(AppSettings::TrailingVarArg)
                                      .arg_from_usage("<cmd>... 'cargo command to run'"))
                      .get_matches();

    let mut cargo_cmd = Command::new(CARGO);
    let mut banner = String::from("Executing ") + CARGO;

    if let Some(arg_cmd) = matches.subcommand_matches("multi")
                                  .and_then(|m| m.values_of("cmd")) {
        for arg in arg_cmd {
            cargo_cmd.arg(arg);
            banner = banner + " " + arg;
        }
    }

    let is_crate = |e: &DirEntry| e.path().join("Cargo.toml").exists();
    let to_path_buf = |e: DirEntry| e.path().to_path_buf();
    let execute = move |p| cargo_cmd.current_dir(p).output().ok();

    if let Ok(cwd) = env::current_dir() {
        announce(&banner);
        walkdir::WalkDir::new(cwd)
            .min_depth(MIN_DEPTH)
            .max_depth(MAX_DEPTH)
            .into_iter()
            .filter_entry(is_crate)
            .filter_map(|e| e.ok())
            .map(to_path_buf)
            .inspect(display_path)
            .filter_map(execute)
            .foreach(report_output);
    }
}
