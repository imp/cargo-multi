# cargo-multi
[![Crates.io](https://img.shields.io/crates/v/cargo-multi.svg?style=plastic)](http://crates.io/crates/cargo-multi)
[![Coverage Status](https://coveralls.io/repos/github/imp/cargo-multi/badge.svg?branch=master)](https://coveralls.io/github/imp/cargo-multi?branch=master)
master: [![Build Status](https://img.shields.io/travis/imp/cargo-multi/master.svg?style=plastic)](https://travis-ci.org/imp/cargo-multi)
develop: [![Build Status](https://img.shields.io/travis/imp/cargo-multi/develop.svg?style=plastic)](https://travis-ci.org/imp/cargo-multi)

Extends cargo to execute the given command on multiple crates. Inspired by `git multi`.

## Installation
Use `cargo` to install this subcommand
```
cargo install cargo-multi
```

## Usage
Run `cargo multi <cargosubcommand>` in the directory where you keep your crates.
```
cargo multi update
```
```
cargo multi build
```
```
cargo multi test
```

## Example
```
$ cargo multi update
----------------------
Executing cargo update
----------------------
cargo:
        Updating registry `https://github.com/rust-lang/crates.io-index`
        Updating git2 v0.3.5 -> v0.3.4
        Removing libgit2-sys v0.4.0
        Updating nom v1.2.0 -> v1.2.1
        Updating num_cpus v0.2.10 -> v0.2.11
        Updating regex v0.1.52 -> v0.1.54
        Updating regex-syntax v0.2.3 -> v0.2.5
        Updating tar v0.4.3 -> v0.4.4
        Removing unicode-bidi v0.2.3
        Removing unicode-normalization v0.1.2
        Removing url v0.5.5

cargo-multi:
        Updating registry `https://github.com/rust-lang/crates.io-index`

elm:
        Updating registry `https://github.com/rust-lang/crates.io-index`

gitlab-rs:
        Updating registry `https://github.com/rust-lang/crates.io-index`

hyper:
        Updating registry `https://github.com/rust-lang/crates.io-index`
        Updating num_cpus v0.2.10 -> v0.2.11
        Updating regex v0.1.52 -> v0.1.54
        Updating regex-syntax v0.2.3 -> v0.2.5
        Updating serde v0.6.14 -> v0.6.15
        Updating unicase v1.2.1 -> v1.3.0

json:
        Updating registry `https://github.com/rust-lang/crates.io-index`
        Updating serde v0.6.14 -> v0.6.15

rass:
        Updating registry `https://github.com/rust-lang/crates.io-index`

requests-rs:
        Updating registry `https://github.com/rust-lang/crates.io-index`
        Updating num_cpus v0.2.10 -> v0.2.11
        Updating serde v0.6.14 -> v0.6.15
        Updating unicase v1.2.1 -> v1.3.0

syncthing-rs:
        Updating registry `https://github.com/rust-lang/crates.io-index`
        Updating regex v0.1.53 -> v0.1.54
        Updating regex-syntax v0.2.4 -> v0.2.5

trust:
        Updating registry `https://github.com/rust-lang/crates.io-index`
        Removing aho-corasick v0.5.1
        Removing docopt v0.6.78
        Removing memchr v0.1.10
        Removing regex v0.1.54
        Removing regex-syntax v0.2.5
        Removing rustc-serialize v0.3.18
        Removing strsim v0.3.0
        Removing utf8-ranges v0.1.3
```
