# mdcat

`mdcat` is a terminal Markdown renderer.

It reads one Markdown file and renders it as readable terminal text using ANSI SGR styling only. It is designed to stay compatible with terminals such as Warp by avoiding interactive terminal control and complex layout behavior.

## Features

- single-file input
- terminal-width wrapping
- ANSI color and emphasis
- theme selection with `--theme dark` or `--theme light`
- default `dark` theme
- no interactive mode

## Usage

```bash
mdcat <file>
mdcat --theme dark <file>
mdcat --theme light <file>
mdcat --no-color <file>
```

Examples:

```bash
cargo run -- sample.md
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

## Sample Document

[`sample.md`](sample.md) contains a broad set of Markdown examples that exercise the renderer and document planned future extensions.

## Development

```bash
cargo test
cargo run -- sample.md
```
