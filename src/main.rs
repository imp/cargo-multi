#[macro_use]
extern crate clap;
extern crate walkdir;
extern crate toml;
extern crate serde_json;

use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::{exit, Command, Output};
use clap::{App, AppSettings, SubCommand};
use walkdir::{DirEntry, WalkDirIterator};


fn announce(banner: &str) {
    let line = "-".repeat(banner.len());
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

fn find_workspaces() -> Result<Option<Vec<PathBuf>>, Box<Error>> {
    let output = Command::new(CARGO)
        .args(&["metadata", "--no-deps", "-q", "--format-version", "1"])
        .output()?;

    if output.status.success() {
        let metadata: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;


        let workspace_members = metadata["packages"]
            .as_array()
            .ok_or("No packages in workspace")?;

        workspace_members
            .iter()
            .map(|package| {
                package["manifest_path"]
                    .as_str()
                    .map(PathBuf::from)
                    .ok_or_else(|| "Invalid manifest path".into())
            })
            .collect::<Result<_, _>>()
            .map(Some)
    } else {
        // If `cargo metadata` fails, it's probably because we're not in a valid workspace.
        Ok(None)
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

fn generate_cargo_cmd(path: &PathBuf, commands: &Vec<String>) -> Command {

    let mut cargo_cmd = Command::new(CARGO);

    // Take a clone of the commands so that the manifest can be passed to the
    // cargo command, this is to any references to files in the output are relative
    // to the current directory.
    let mut commands = commands.clone();

    commands.insert(1, "--manifest-path".to_string());
    commands.insert(2, format!("{}/Cargo.toml", path.to_string_lossy()));

    for arg in commands {
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
        .subcommand(
            SubCommand::with_name("multi")
                .version(crate_version!())
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::TrailingVarArg)
                .arg_from_usage("<cmd>... 'cargo command to run'"),
        )
        .get_matches();

    let commands = matches.subcommand_matches("multi")
                          .and_then(|m| m.values_of("cmd"))
                          .expect("No cargo commands provided")
                          .map(|arg| arg.to_string())
                          .collect::<Vec<_>>();

    let banner = format!("Executing {} {}", CARGO, commands.join(" "));

    let banner = banner.join(" ");

    announce(&banner);

    let dirs = find_workspaces()
        .expect("Failed to get workspace members")
        .unwrap_or_else(find_crates);

    let display_path = |p: &PathBuf| println!("{}:", p.to_string_lossy());
    let execute = |p: PathBuf| generate_cargo_cmd(&p, &commands).output().ok();

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
