# iOS-Backup

[![made-with-rust][rust-logo]][rust-src-page]

[![crates.io][crates-logo]][crate]

[![build][gh-logo]][build]
[![none-shall-pass][nsp-logo]][nsp]

#### Summary
[`ios`][repo] is a lightweight CLI tool to extract iPhone backups.

### Installation

```shell
cargo add ios
```

### Usage

**Embedded**
```rust
use ios;

fn main() {
    match ios::extractor() {
        Ok(res) => {
            println!("{}", res);
        }
        Err(err) => {
            println!("{}", err);
        }
    };
}
```

**CLI**
```
./ios
```

`ios` crate takes the following arguments

- `--version` - Print project version.
- `--list` - List the available backups.
- `--debug` - Enable debug level logging.
- `--all` - Extract all available backups.
- `--serial` - Initiate backup extraction for given serial number(s).
- `--organize` - Organize the extracted files by `type`, `size`, `root`, and `auto`.
- `--workers` | `--threads` - Numbers of workers (threads) to spin up for extraction.
- `--backup-dir` | `--source` - Custom path for the backup. Defaults to OS specific path.
- `--output-dir` | `--destination` - Destination directory. Defaults to 'extracted' in current path.

<br>

<details>
<summary><strong>Download OS specific Executable</strong></summary>

###### macOS (x86)
```shell
curl -o ios-Darwin-x86_64.tar.gz -LH "Accept: application/octet-stream" "https://github.com/thevickypedia/ios/releases/latest/download/ios-Darwin-x86_64.tar.gz"
```

###### macOS (ARM)
```shell
curl -o ios-Darwin-arm64.tar.gz -LH "Accept: application/octet-stream" "https://github.com/thevickypedia/ios/releases/latest/download/ios-Darwin-arm64.tar.gz"
```

###### Linux
```shell
curl -o ios-Linux-x86_64.tar.gz -LH "Accept: application/octet-stream" "https://github.com/thevickypedia/ios/releases/latest/download/ios-Linux-x86_64.tar.gz"
```

###### Windows
```shell
curl -o ios-Windows-x86_64.zip -LH "Accept: application/octet-stream" "https://github.com/thevickypedia/ios/releases/latest/download/ios-Windows-x86_64.zip"
```
</details>

## Crate
[https://crates.io/crates/ios][crate]

### Cargo Docs - Official Runbook
[https://docs.rs/ios/latest/ios/][docs]

**Generator**
```shell
cargo doc --document-private-items --no-deps
```

## Linting
### Requirement
```shell
rustup component add clippy
```
### Usage
```shell
cargo clippy --no-deps --fix
```

## License & copyright

&copy; Vignesh Rao

Licensed under the [MIT License][license]

[repo]: https://github.com/thevickypedia/iOS-Backup
[license]: https://github.com/thevickypedia/iOS-Backup/blob/main/LICENSE
[build]: https://github.com/thevickypedia/iOS-Backup/actions/workflows/rust.yml
[rust-src-page]: https://www.rust-lang.org/
[rust-logo]: https://img.shields.io/badge/Made%20with-Rust-black?style=for-the-badge&logo=Rust
[gh-logo]: https://github.com/thevickypedia/iOS-Backup/actions/workflows/rust.yml/badge.svg
[nsp-logo]: https://github.com/thevickypedia/iOS-Backup/actions/workflows/none.yml/badge.svg
[nsp]: https://github.com/thevickypedia/iOS-Backup/actions/workflows/none.yml
[crate]: https://crates.io/crates/ios
[gh-checks]: https://github.com/thevickypedia/iOS-Backup/actions/workflows/rust.yml
[crates-logo]: https://img.shields.io/crates/v/ios.svg
[gh-wiki]: https://github.com/thevickypedia/iOS-Backup/wiki
[gh-wiki-env]: https://github.com/thevickypedia/iOS-Backup/wiki/Environment-Variables
[docs]: https://docs.rs/ios/latest/ios/
