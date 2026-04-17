# mdcat Sample

This document is a compact test corpus for `mdcat`.

It includes the Markdown features supported in v1, plus the features planned for later phases so renderer behavior can be checked against a single file.

## Headings

### Level 3 Heading

#### Level 4 Heading

##### Level 5 Heading

###### Level 6 Heading

## Inline Formatting

This paragraph contains *italic text*, **bold text**, and `inline code`.

You can also combine styles like **bold and *nested italic*** in the same sentence.

Links should stay readable, like [the Rust book](https://doc.rust-lang.org/book/) and [Markdown Guide](https://www.markdownguide.org/).

## Paragraph Wrapping

This is a longer paragraph meant to exercise terminal-width wrapping. The renderer should keep the text readable without using cursor movement or other complex terminal control sequences. The wrapping should happen naturally based on the current terminal width.

## Lists

- Unordered item one
- Unordered item two with a longer description that should wrap cleanly onto the next line
  - Nested unordered item
  - Another nested item

1. Ordered item one
2. Ordered item two with a longer description that should also wrap cleanly onto the next line
3. Ordered item three

## Blockquotes

> A quote can span multiple lines and should remain visually distinct.
>
> Another quoted paragraph keeps the blockquote structure intact.

## Code

Inline code like `cargo run -- sample.md` should be styled differently from normal text.

```rust
fn main() {
    let title = "mdcat";
    println!("Rendering {title} from Markdown.");
}
```

```bash
cargo run -- --theme light sample.md
cargo run -- --theme dark sample.md
```

## Horizontal Rule

---

## Future Features

The following examples are intentionally included for features planned in a later phase.

### Table

| Feature | Status | Notes |
| --- | --- | --- |
| Tables | Planned | Render in aligned columns later |
| Footnotes | Planned | Attach references and definitions |
| Task lists | Planned | Show checkbox states |
| Images | Planned | Display alt text or a placeholder |

### Footnote

Here is a sentence with a footnote reference.[^1]

[^1]: This is the footnote definition that will be supported later.

### Task List

- [ ] Design table renderer
- [x] Keep Warp-compatible output
- [ ] Add footnote rendering

### Image

![mdcat logo](assets/mdcat.png)

An image reference is included for future rendering work.

## Mixed Content

> **Note:** `mdcat` should stay linear and predictable.
>
> It should not rely on alternate screens, cursor positioning, or interactive output.

## Final Paragraph

The sample file is meant to be broad enough to catch regressions in headings, inline styles, wrapping, lists, blockquotes, and code blocks, while also documenting the planned v2 features in one place.
