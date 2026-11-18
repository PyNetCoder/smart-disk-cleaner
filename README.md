# Smart Disk Cleaner

A small command-line tool that finds common generated developer directories, reports their approximate disk usage, and removes them only after you confirm.

## What it scans

The scanner looks recursively for directories named:

- `node_modules`
- `target`
- `.venv`
- `build`

It does not follow symbolic links. The reported size is the sum of regular file sizes below each matching directory. It is an estimate of reclaimable space and may differ from filesystem usage because of hard links, sparse files, compression, and filesystem metadata.

## Build

Install a recent Rust toolchain, then run:

```sh
cargo build --release
```

## Usage

Scan the current directory and its subdirectories:

```sh
cargo run --release
```

Scan a specific directory:

```sh
cargo run --release -- /path/to/workspace
```

On Windows, a path can be supplied with normal Windows syntax:

```powershell
cargo run --release -- "C:\Users\you\source"
```

The tool lists every match and its estimated size, then prompts before deleting anything. Enter `y` or `yes` to remove the listed directories. Any other response leaves them in place. Use `--help` to display command usage.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
