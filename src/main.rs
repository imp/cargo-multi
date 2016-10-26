#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate clap;
extern crate itertools;
extern crate walkdir;
extern crate toml;

use std::env;
use std::fs::File;
use std::io::Read;
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

    // I am still not sure what is more idiomatic - the 'if' above or the 'match' below
    //
    // match output.status.success() {
    //     true => print_ident(output.stdout),
    //     false => print_ident(output.stderr),
    // }
    println!("");

    output.status
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
                      .subcommand(SubCommand::with_name("multi")
                                      .version(crate_version!())
                                      .setting(AppSettings::ArgRequiredElseHelp)
                                      .setting(AppSettings::TrailingVarArg)
                                      .arg_from_usage("<cmd>... 'cargo command to run'"))
                      .get_matches();

    let commands = matches.subcommand_matches("multi")
                          .and_then(|m| m.values_of("cmd"))
                          .expect("No cargo commands provided")
                          .map(|arg| arg.to_string())
                          .collect::<Vec<_>>();

    let banner = format!("Executing {} {}", CARGO, commands.join(" "));

    announce(&banner);
    let is_crate = |e: &DirEntry| e.path().join("Cargo.toml").exists();
    let display_path = |p: &PathBuf| println!("{}:", p.to_string_lossy());
    let execute = |p: PathBuf| generate_cargo_cmd(&p, &commands).output().ok();

    // First check if there is a Cargo.toml file with a workspace section in.
    let mut workspace_members = match File::open("Cargo.toml") {
        Ok(mut file) => {
            let mut toml = String::new();
            match file.read_to_string(&mut toml) {
                Ok(_) => {
                    let value: toml::Value = toml.parse().expect("Failed to parse Cargo.toml");

                    match value.lookup("workspace.members") {
                        Some(members) => {
                            Some(members.as_slice()
                                        .expect("Failed to read workspace members")
                                        .into_iter()
                                        .map(|m| PathBuf::from(m.as_str().unwrap()))
                                        .collect::<Vec<_>>())
                        }
                        None => None,
                    }
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    };

    // If there was no workspace members present, add each crate directory
    // present.
    if workspace_members.is_none() {
        workspace_members = match env::current_dir() {
            Ok(cwd) => {
                Some(walkdir::WalkDir::new(cwd)
                                .min_depth(MIN_DEPTH)
                                .max_depth(MAX_DEPTH)
                                .into_iter()
                                .filter_entry(is_crate)
                                .filter_map(|e| e.ok())
                                .map(|m| m.path().to_path_buf())
                                .collect::<Vec<_>>())
            }
            Err(_) => None,
        }
    }

    let failed_commands = match workspace_members {
        Some(members) => {
            members.into_iter()
                   .inspect(display_path)
                   .filter_map(execute)
                   .map(report_output)
                   .filter(|x| !x.success())
                   .collect::<Vec<_>>()
        }
        None => Vec::new(),
    };

    // If there are any failed commands, return the error code of the
    // first of them.
    if failed_commands.len() > 0 {
        exit(failed_commands[0].code().unwrap());
    }
}
