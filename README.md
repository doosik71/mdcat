# mdcat

`mdcat` is a terminal Markdown renderer.

It can read one Markdown file or stream Markdown from standard input and renders it as readable terminal text using ANSI SGR styling only. It is designed to stay compatible with terminals such as Warp by avoiding interactive terminal control and complex layout behavior.

## Features

- single-file input
- stdin streaming input
- terminal-width wrapping
- ANSI color and emphasis
- theme selection with `--theme dark` or `--theme light`
- default `dark` theme
- no interactive mode

## Usage

```bash
mdcat <file>
mdcat
cat file.md | mdcat
mdcat --theme dark <file>
mdcat --theme light <file>
mdcat --no-color <file>
```

Examples:

```bash
cargo run -- sample.md
cargo run --
cargo run -- < sample.md
cargo run -- --theme light sample.md
cargo run -- --theme dark sample.md
```

## Supported Markdown

v1 support includes:

- headings
- paragraphs
- emphasis and strong emphasis
- inline code
- fenced code blocks
- unordered lists
- ordered lists
- blockquotes
- links
- tables
- task lists
- images
- footnotes

## Theme Notes

The renderer supports two explicit themes:

- `dark`
- `light`

The default is `dark`. If you want to change colors, edit the element palette constants near the top of [`src/lib.rs`](src/lib.rs).

## Input Modes

`mdcat` supports two separate rendering paths:

- file mode uses the existing document renderer unchanged
- stdin mode uses a separate block-oriented streaming pipeline that reads blocks from stdin and writes rendered output directly to stdout

The streaming path is intentionally isolated so the file renderer stays stable.

## Sample Document

[`sample.md`](sample.md) contains a broad set of Markdown examples that exercise the renderer and document planned future extensions.

## Development

```bash
cargo test
cargo run -- sample.md
```

## Release Build

Build an optimized binary with:

```bash
cargo build --release
```

The binary will be available at:

```bash
target/release/mdcat
```

## Installation

Install `mdcat` into your Cargo bin directory with:

```bash
cargo install --path .
```

This places the executable in Cargo's bin directory, typically:

```bash
~/.cargo/bin/mdcat
```

If you prefer to install manually after a release build, copy the binary from `target/release/mdcat` to a directory on your `PATH`.
