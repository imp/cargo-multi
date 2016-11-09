#[macro_use]
extern crate clap;
extern crate walkdir;
extern crate toml;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use clap::{App, SubCommand, AppSettings};
use walkdir::{DirEntry, WalkDirIterator};


fn announce(banner: &str) {
    let mut line = String::new();
    for _ in 0..banner.len() {
        line.push('-');
    }
    println!("{}\n{}\n{}", line, banner, line);
}

fn read_file<P: AsRef<Path>>(path: P) -> Option<String> {
    File::open(path)
        .and_then(|mut f| {
            let mut t = String::new();
            f.read_to_string(&mut t).map(|_| t)
        })
        .ok()
}

fn find_workspaces() -> Option<Vec<PathBuf>> {
    if let Some(ref toml) = read_file("Cargo.toml").and_then(|t| t.parse::<toml::Value>().ok()) {
        toml.lookup("workspace.members")
            .and_then(|w| w.as_slice())
            .map(|v| {
                v.into_iter()
                    .filter_map(|s| s.as_str())
                    .map(PathBuf::from)
                    .collect::<Vec<_>>()
            })
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

fn generate_cargo_cmd(path: &PathBuf, commands: &[String], use_manifest: bool) -> Command {

    let mut cargo_cmd = Command::new(CARGO);

    let (command, args) = commands.split_at(1);

    cargo_cmd.arg(command[0].clone());

    // If a manifest file is required to be be passed to the subcommand do it
    // now. Otherwise, we just set the current directory.
    if use_manifest {
        // Clippy requires the full path to be provided for the manifest-path, so
        // pass it across in full.
        let full_path = std::fs::canonicalize(path).expect("Failed to expand path");

        // Insert the manifest-path option so that any logs about files are relative
        // to the current directory.
        cargo_cmd.arg(format!("--manifest-path={}/Cargo.toml", full_path.to_string_lossy()));
    } else {
        cargo_cmd.current_dir(path);
    }

    for arg in args {
        cargo_cmd.arg(arg);
    }

    cargo_cmd
}

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

            .arg_from_usage("--manifest  'Adds --manifest-path to command to run'")
            .arg_from_usage("<cmd>... 'cargo command to run'"))
        .get_matches();

    let multi_subcommand = matches.subcommand_matches("multi").expect("multi command was not provided");
    let use_manifest = multi_subcommand.is_present("manifest");

    let commands = multi_subcommand.values_of("cmd")
        .expect("No cargo commands provided")
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>();

    let banner = format!("Executing {} {}", CARGO, commands.join(" "));

    announce(&banner);

    let dirs = find_workspaces().unwrap_or_else(find_crates);

    let display_path = |p: &PathBuf| println!("\n{}:", p.to_string_lossy());
    let execute = move |p: PathBuf| generate_cargo_cmd(&p, &commands, use_manifest).status().ok();

    let failed_commands = dirs.into_iter()
        .inspect(display_path)
        .filter_map(execute)
        .filter(|x| !x.success())
        .collect::<Vec<_>>();

    // If there are any failed commands, return the error code of the
    // first of them.
    if !failed_commands.is_empty() {
        exit(failed_commands[0].code().unwrap());
    }
}
