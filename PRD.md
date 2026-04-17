# mdcat PRD

## 1. Overview

`mdcat` is a terminal Markdown renderer.

It behaves like `cat` for a single file input, but renders Markdown in a readable terminal-friendly form rather than printing raw source text.

The design goal is to stay compatible with terminals such as Warp by limiting output to plain text plus ANSI SGR styling only. `mdcat` must not use interactive terminal modes, cursor movement, alternate screens, or other complex terminal control sequences.

## 2. Problem Statement

Common Markdown renderers sometimes rely on terminal behavior that is not consistent across all terminal emulators. Warp in particular splits output into blocks, which can break renderers that depend on complex layout control or interactive rendering.

`mdcat` solves this by:

- reading exactly one Markdown file
- rendering it as a linear terminal stream
- using ANSI escape codes only for color and emphasis
- wrapping output to the current terminal width
- avoiding any terminal control beyond simple styled text output

## 3. Goals

- Accept one file path as input.
- Render Markdown to readable terminal text.
- Support terminal-width wrapping for paragraphs and list items.
- Use ANSI SGR styling for emphasis such as bold, italic, code, and links.
- Stay compatible with Warp by avoiding advanced terminal control.
- Keep the implementation simple, deterministic, and easy to test.

## 4. Non-Goals

- No interactive mode.
- No pager integration.
- No stdin input in v1.
- No multi-file rendering.
- No terminal UI.
- No support for tables, footnotes, task lists, or images in v1.

## 5. v1 Markdown Scope

Supported in the first version:

- headings
- paragraphs
- emphasis and strong emphasis
- inline code
- fenced code blocks
- unordered lists
- ordered lists
- blockquotes
- links

Deferred to v2:

- tables
- footnotes
- task lists
- images

## 6. CLI Contract

Primary usage:

```bash
mdcat <file>
```

Optional theme selection:

```bash
mdcat --theme dark <file>
mdcat --theme light <file>
```

Behavior:

- print a usage error if no file is provided
- fail if the file cannot be read
- render the file contents to stdout
- wrap output to terminal width when width can be determined
- default to the `dark` theme
- accept an explicit `--theme light` or `--theme dark` value
- disable color when stdout is not a terminal or when `--no-color` is used

## 7. Rendering Rules

### 7.1 General

- Output is a single linear stream.
- Use only ANSI SGR codes for style.
- Reset styles after styled spans.
- Do not use cursor movement, screen clearing, raw mode, or alternate screen.
- Theme selection is explicit and deterministic.
- `dark` is the default theme.
- `light` uses a palette better suited for light terminal backgrounds.

### 7.2 Headings

- Render headings with stronger emphasis than normal text.
- Use bold styling and a distinct color.
- Leave a blank line before and after headings.

### 7.3 Paragraphs

- Wrap to terminal width.
- Preserve blank lines between blocks.
- Apply inline styling within the wrapped text.

### 7.4 Lists

- Render list markers plainly.
- Indent nested content relative to the marker.
- Wrap continuation lines under the text, not under the bullet or number.
- Task list markers should be rendered with theme-aware checked/unchecked colors.

### 7.5 Blockquotes

- Prefix each quoted line with a visible quote marker.
- Keep the quote text wrapped within the available width.
- Apply a theme-aware background color across the full blockquote block.
- Render the `>` marker in a separate foreground color.

### 7.6 Code Blocks

- Render fenced code blocks in a monospace-friendly plain layout.
- Preserve line breaks from the source.
- Style code blocks with a muted color.
- Apply a theme-aware background color across the full code block.

### 7.7 Links

- Render visible link text normally.
- Optionally add the URL in parentheses when helpful, without breaking wrapping.

### 7.8 Tables, Images, and Deferred Elements

- Keep pipe tables visible as aligned text.
- Apply a theme-aware foreground color to table text.
- Render table headers in bold.
- Render images as `![alt](url)` and apply a theme-aware foreground color to the full image span.
- Defer footnotes to later phases.

## 8. Implementation Plan

### Phase 1

- Create a reusable Markdown rendering module.
- Parse Markdown into a stream or structured representation.
- Implement ANSI styling helpers.
- Implement word wrapping aware of indentation and prefixes.
- Add a CLI that reads one file and prints rendered output.
- Add explicit `--theme light|dark` selection with `dark` as default.
- Add tests for common Markdown samples.

### Phase 2

- Add support for tables, footnotes, task lists, and images if needed.
- Expand theme and styling options.
- Improve edge cases in wrapping and nested formatting.

## 9. Acceptance Criteria

`mdcat` is considered ready for v1 when:

- `mdcat path/to/file.md` renders the file successfully
- output uses ANSI SGR only
- text wraps to the terminal width
- `--theme dark` is the default when no theme is provided
- headings, paragraphs, lists, blockquotes, links, and code blocks render sensibly
- tables, footnotes, task lists, and images are explicitly out of scope for v1

## 10. Risks

- Markdown nesting can complicate inline styling and wrapping.
- Terminal width detection can vary by environment.
- Different terminals may render ANSI color intensity differently.
- Warp compatibility must be preserved by keeping output strictly linear.
