extern crate walkdir;

use std::ffi::OsString;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::vec::IntoIter;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

struct MultiCmd {
    cmd: String,
    args: Vec<String>,
    min_depth: usize,
    max_depth: usize,
    root: PathBuf,
}

struct MultiCmdRunner {
    command: Command,
    directories: IntoIter<PathBuf>,
}

impl MultiCmd {
    fn new(cmd: &str, args: Vec<String>) -> Self {

        MultiCmd {
            cmd: String::from(cmd),
            args: args,
            min_depth: 1,
            max_depth: 1,
            root: PathBuf::new(),
        }
    }

    fn rootdir(&mut self, dir: PathBuf) -> &mut Self {
        self.root = dir;
        self
    }

    fn build(&self) -> MultiCmdRunner {
        let mut command = Command::new(&self.cmd);
        let mut banner = String::from("Executing ") + &self.cmd;
        for arg in &self.args {
            command.arg(OsString::from(&arg));
            banner = banner + " " + &arg;
        }
        let dirs = WalkDir::new(&self.root)
                       .min_depth(self.min_depth)
                       .max_depth(self.max_depth)
                       .into_iter()
                       .filter_entry(|e| is_crate(e))
                       .map(|e| e.ok().unwrap().path().to_path_buf())
                       .collect::<Vec<_>>();

        announce(&banner);

        MultiCmdRunner {
            command: command,
            directories: dirs.into_iter(),
        }
    }
}

impl MultiCmdRunner {
    fn run(&mut self, dir: PathBuf) -> Output {
        println!("{}:", dir.display());
        self.command.current_dir(dir).output().unwrap()
    }
}

impl Iterator for MultiCmdRunner {
    type Item = Output;
    fn next(&mut self) -> Option<Output> {
        match self.directories.next() {
            Some(p) => Some(self.run(p)),
            None => None,
        }
    }
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
    // ::<'a>(v: &'a [u8])
    for line in String::from_utf8_lossy(&buf[..]).lines() {
        println!("        {}", line);
    }
}

fn is_crate(entry: &DirEntry) -> bool {
    entry.path().join("Cargo.toml").exists()
}

fn main() {
    let args = env::args().skip(2).collect();
    let cwd = env::current_dir().unwrap();
    let multi = MultiCmd::new("cargo", args)
                        .rootdir(cwd)
                        .build();

    for result in multi {
        if result.status.success() {
            print_ident(result.stdout);
        } else {
            print_ident(result.stderr);
        }
        println!("");

    }
}
