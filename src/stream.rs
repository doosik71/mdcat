use std::io::{self, BufRead, Write};

use crate::{render_markdown, RenderConfig, Theme};

pub fn render_streaming<R: BufRead, W: Write>(
    mut input: R,
    mut output: W,
    color: bool,
    theme: Theme,
) -> io::Result<()> {
    let config = RenderConfig::from_terminal(color, theme);
    let mut block = String::new();
    let mut first_block = true;
    let mut in_fence: Option<String> = None;
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = input.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);
        let is_blank = trimmed.trim().is_empty();

        if let Some(fence) = in_fence.as_ref() {
            block.push_str(&line);
            if fence_close(trimmed, fence) {
                flush_block(&mut output, &mut block, &mut first_block, config)?;
                in_fence = None;
            }
            continue;
        }

        if is_blank {
            flush_block(&mut output, &mut block, &mut first_block, config)?;
            continue;
        }

        if let Some(fence) = fence_start(trimmed) {
            flush_block(&mut output, &mut block, &mut first_block, config)?;
            in_fence = Some(fence);
            block.push_str(&line);
            continue;
        }

        block.push_str(&line);
    }

    flush_block(&mut output, &mut block, &mut first_block, config)?;
    output.write_all(b"\n")?;
    output.flush()
}

fn flush_block<W: Write>(
    output: &mut W,
    block: &mut String,
    first_block: &mut bool,
    config: RenderConfig,
) -> io::Result<()> {
    if block.trim().is_empty() {
        block.clear();
        return Ok(());
    }

    if !*first_block {
        output.write_all(b"\n\n")?;
    }

    let rendered = render_markdown(block, config);
    output.write_all(rendered.as_bytes())?;
    *first_block = false;
    block.clear();
    Ok(())
}

fn fence_start(line: &str) -> Option<String> {
    let stripped = line.trim_start();
    let fence = stripped.strip_prefix("```").map(|_| "```")
        .or_else(|| stripped.strip_prefix("~~~").map(|_| "~~~"))?;
    Some(fence.to_string())
}

fn fence_close(line: &str, fence: &str) -> bool {
    let stripped = line.trim_start();
    stripped.starts_with(fence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn streams_multiple_blocks() {
        let input = Cursor::new(b"# Title\n\nHello world.\n".to_vec());
        let mut output = Vec::new();

        render_streaming(input, &mut output, false, Theme::Dark).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("# Title"));
        assert!(rendered.contains("Hello world."));
        assert!(rendered.contains("\n\n"));
        assert!(rendered.ends_with('\n'));
    }

    #[test]
    fn streams_fenced_code_block() {
        let input = Cursor::new(b"```rust\nfn main() {}\n```\n".to_vec());
        let mut output = Vec::new();

        render_streaming(input, &mut output, false, Theme::Dark).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("fn main() {}"));
        assert!(rendered.ends_with('\n'));
    }
}
