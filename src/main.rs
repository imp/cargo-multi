extern crate walkdir;

use std::ffi::OsString;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

struct MultiCmd {
    command: Command,
    banner: String,
    top_directory: PathBuf,
    directories: Vec<PathBuf>,
}

impl MultiCmd {
    fn new(cmd: &str, args: Vec<String>) -> Self {

        let mut command = Command::new(cmd);
        let mut banner = String::from("Executing ") + &cmd ;
        for arg in args {
            command.arg(OsString::from(&arg));
            banner = banner + " " + &arg;
        }

        MultiCmd {
            command: command,
            banner: banner,
            top_directory: PathBuf::new(),
            directories: Vec::new(),
        }
    }

    fn set_top_dir(&mut self, dir: PathBuf) -> &mut Self {
        self.top_directory = dir;
        self
    }

    fn collect_crate_candidates(&mut self) -> &mut Self {
        self.directories = WalkDir::new(&self.top_directory)
                               .min_depth(1)
                               .max_depth(1)
                               .into_iter()
                               .filter_entry(|e| is_crate(e))
                               .map(|e| e.ok().unwrap().path().to_path_buf())
                               .collect::<Vec<_>>();
        self
    }

    fn announce(&self) -> &Self {
        let mut line = String::new();
        for _ in 0..self.banner.len() {
            line.push('-');
        }
        println!("{}", line);
        println!("{}", self.banner);
        println!("{}", line);
        self
    }

    fn dispatch(&mut self) -> &Self {
        for dir in &self.directories {
            println!("{}:", dir.display());
            let out = self.command
                          .current_dir(dir)
                          .output()
                          .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
            if out.status.success() {
                print_ident(out.stdout);
            } else {
                print_ident(out.stderr);
            }
            println!("");
        }
        self
    }
}

fn print_ident(buf: Vec<u8>)
{
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
    let mut multi = MultiCmd::new("cargo", args);

    multi.set_top_dir(cwd);
    multi.collect_crate_candidates();
    multi.announce();
    multi.dispatch();
}
