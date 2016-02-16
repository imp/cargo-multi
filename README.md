# cargo-multi
[![Crates.io](https://img.shields.io/crates/v/cargo-multi.svg?style=plastic)](http://crates.io/crates/cargo-multi)
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
/Users/cyrilp/dev/rust/bep-rs:
        Updating registry `https://github.com/rust-lang/crates.io-index`
        Updating aho-corasick v0.4.1 -> v0.5.0
        Updating regex v0.1.48 -> v0.1.51
        Updating regex-syntax v0.2.2 -> v0.2.3
          Adding utf8-ranges v0.1.3

/Users/cyrilp/dev/rust/cargo-multi:
        Updating registry `https://github.com/rust-lang/crates.io-index`

/Users/cyrilp/dev/rust/elm:
        Updating registry `https://github.com/rust-lang/crates.io-index`

/Users/cyrilp/dev/rust/gitlab-rs:
        Updating registry `https://github.com/rust-lang/crates.io-index`
        Updating gcc v0.3.23 -> v0.3.24

/Users/cyrilp/dev/rust/hyper:
        Updating registry `https://github.com/rust-lang/crates.io-index`

/Users/cyrilp/dev/rust/json:
        Updating registry `https://github.com/rust-lang/crates.io-index`

/Users/cyrilp/dev/rust/rass:
        Updating registry `https://github.com/rust-lang/crates.io-index`

/Users/cyrilp/dev/rust/requests-rs:
        Updating registry `https://github.com/rust-lang/crates.io-index`
        Updating gcc v0.3.23 -> v0.3.24

/Users/cyrilp/dev/rust/trust:
        Updating registry `https://github.com/rust-lang/crates.io-index`
```
