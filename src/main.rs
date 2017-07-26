#[macro_use]
extern crate clap;
extern crate walkdir;
extern crate toml;
extern crate serde_json;

use std::env;
use std::path::PathBuf;
use std::process::{exit, Command, Output};
use clap::{App, SubCommand, AppSettings};
use walkdir::{DirEntry, WalkDirIterator};


fn announce(banner: &str) {
    let mut line = String::new();
    for _ in 0..banner.len() {
        line.push('-');
    }
    println!("{}\n{}\n{}", line, banner, line);
}

fn print_ident(buf: Vec<u8>) {
    for line in String::from_utf8_lossy(&buf[..]).lines() {
        println!("        {}", line);
    }
}

fn report_output(output: Output) -> std::process::ExitStatus {
    if output.status.success() {
        print_ident(output.stdout);
    }

    // Always print stderr as warnings from cargo are sent to stderr.
    print_ident(output.stderr);
    println!("");
    output.status
}

fn find_workspaces() -> Option<Vec<PathBuf>> {
    let output = Command::new(CARGO)
        .args(&["metadata", "--no-deps", "-q", "--format-version",  "1"])
        .output()
        .expect("Failed to run `cargo metadata`");

    if output.status.success() {
        let metadata: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
                .expect("Invalid cargo metadata");

        metadata
            .get("workspace_members")
            .and_then(|members| members.as_array())
            .map(|members| members.iter()
                                  .filter_map(|member| member.as_str())
                                  .map(|path| path.trim_left_matches("path+file://"))
                                  .map(PathBuf::from).collect())
    } else {
        None
    }
}

fn find_crates() -> Vec<PathBuf> {
    let is_crate = |e: &DirEntry| e.path().join("Cargo.toml").exists();

    if let Ok(cwd) = env::current_dir() {
        walkdir::WalkDir::new(cwd)
            .min_depth(MIN_DEPTH)
            .max_depth(MAX_DEPTH)
            .into_iter()
            .filter_entry(is_crate)
            .filter_map(|e| e.ok())
            .map(|m| m.path().to_path_buf())
            .collect::<Vec<_>>()
    } else {
        vec![]
    }
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
    let mut banner = vec!["Executing", CARGO];

    if let Some(arg_cmd) = matches.subcommand_matches("multi").and_then(|m| m.values_of("cmd")) {
        for arg in arg_cmd {
            cargo_cmd.arg(arg);
            banner.push(arg);
        }
    }

    let banner = banner.join(" ");

    announce(&banner);

    let dirs = find_workspaces().unwrap_or_else(find_crates);

    let display_path = |p: &PathBuf| println!("{}:", p.to_string_lossy());
    let execute = |p: PathBuf| cargo_cmd.current_dir(p).output().ok();

    let failed_commands = dirs.into_iter()
        .inspect(display_path)
        .filter_map(execute)
        .map(report_output)
        .filter(|x| !x.success())
        .collect::<Vec<_>>();

    // If there are any failed commands, return the error code of the
    // first of them.
    if !failed_commands.is_empty() {
        exit(failed_commands[0].code().unwrap());
    }
}
