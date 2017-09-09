#[macro_use]
extern crate clap;
extern crate walkdir;
extern crate toml;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{exit, Command, Output};
use clap::{App, SubCommand, AppSettings};
use walkdir::{DirEntry, WalkDirIterator};


fn announce(cargo: &str, commands: &[String]) {
    let banner = format!("Executing {} {}", cargo, commands.join(" "));
    let line = "-".repeat(banner.len());
    println!("{}\n{}\n{}", line, banner, line);
}

fn print_ident(buf: &[u8]) {
    for line in String::from_utf8_lossy(buf).lines() {
        println!("        {}", line);
    }
}

fn report_output(output: &Output) {
    if output.status.success() {
        print_ident(&output.stdout);
    }

    // Always print stderr as warnings from cargo are sent to stderr.
    print_ident(&output.stderr);
    println!("");
}

fn load_file<P: AsRef<Path>>(path: P) -> Option<toml::Value> {
    let mut text = String::new();
    let f = File::open(path);
    if f.is_ok() {
        f.unwrap().read_to_string(&mut text);
    }
    text.parse::<toml::Value>().ok()
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
        toml.get("workspace.members")
            .and_then(|w| w.as_array())
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

const MIN_DEPTH: usize = 1;
const MAX_DEPTH: usize = 1;

fn generate_cargo_cmd(cargo: &str, path: &Path, commands: &[String]) -> Command {

    let mut cargo_cmd = Command::new(cargo);

    let (command, args) = commands.split_at(1);

    cargo_cmd.arg(command[0].clone());

    // Insert the manifest-path option so that any logs about files are relative
    // to the current directory.
    cargo_cmd.arg("--manifest-path".to_string());
    cargo_cmd.arg(path.join("Cargo.toml"));

    for arg in args {
        cargo_cmd.arg(arg);
    }

    cargo_cmd
}

fn main() {

    let cargo = env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));

    let matches = App::new(cargo.as_str())
        .bin_name(cargo.as_str())
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

    let commands = matches
        .subcommand_matches("multi")
        .and_then(|m| m.values_of("cmd"))
        .expect("No cargo commands provided")
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>();

    announce(&cargo, &commands);

    let dirs = find_workspaces().unwrap_or_else(find_crates);

    let display_path = |p: &PathBuf| println!("{}:", p.display());
    let execute = |path: PathBuf| generate_cargo_cmd(&cargo, &path, &commands).output().ok();

    let failed_commands = dirs.into_iter()
        .inspect(display_path)
        .filter_map(execute)
        .inspect(report_output)
        .filter(|x| !x.status.success())
        .collect::<Vec<_>>();

    // If there are any failed commands, return the error code of the
    // first of them.
    if !failed_commands.is_empty() {
        exit(failed_commands[0].status.code().unwrap());
    }
}
