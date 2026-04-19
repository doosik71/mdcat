use std::fs;
use std::io;
use std::path::Path;

use pulldown_cmark::{Alignment, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use terminal_size::{terminal_size, Width};
use unicode_width::UnicodeWidthChar;

mod stream;
pub use stream::render_streaming;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const ITALIC: &str = "\x1b[3m";
const UNDERLINE: &str = "\x1b[4m";
const DIM: &str = "\x1b[2m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const MAGENTA: &str = "\x1b[35m";
const YELLOW: &str = "\x1b[33m";
// const WHITE: &str = "\x1b[37m";
const BLACK: &str = "\x1b[30m";
const BRIGHT_BLACK: &str = "\x1b[90m";
const QUOTE_BG_DARK: &str = "\x1b[48;2;37;41;49m";
const QUOTE_BG_LIGHT: &str = "\x1b[48;2;239;242;247m";
const CODE_BG_DARK: &str = "\x1b[48;2;27;31;38m";
const CODE_BG_LIGHT: &str = "\x1b[48;2;245;246;248m";
// const BG_BLUE: &str = "\x1b[44m";
const BG_MAGENTA: &str = "\x1b[45m";
const BG_GREEN: &str = "\x1b[42m";
// const BG_PINK: &str = "\x1b[48;5;213m";

// Element palette: edit these constants to change mdcat colors in one place.
const HEADING_1_DARK_FG: &str = GREEN;
const HEADING_1_DARK_BG: &str = BG_MAGENTA;
const HEADING_2_DARK_FG: &str = BLUE;
const HEADING_3_DARK_FG: &str = GREEN;
const HEADING_4_DARK_FG: &str = MAGENTA;
const HEADING_5_DARK_FG: &str = CYAN;
const HEADING_6_DARK_FG: &str = BLUE;

const HEADING_1_LIGHT_FG: &str = MAGENTA;
const HEADING_1_LIGHT_BG: &str = BG_GREEN;
const HEADING_2_LIGHT_FG: &str = BRIGHT_BLACK;
const HEADING_3_LIGHT_FG: &str = BLUE;
const HEADING_4_LIGHT_FG: &str = BLACK;
const HEADING_5_LIGHT_FG: &str = BRIGHT_BLACK;
const HEADING_6_LIGHT_FG: &str = BRIGHT_BLACK;

const LINK_DARK_FG: &str = BLUE;
const LINK_LIGHT_FG: &str = BRIGHT_BLACK;

const INLINE_CODE_DARK_FG: &str = CYAN;
const INLINE_CODE_LIGHT_FG: &str = BLACK;
const INLINE_CODE_DARK_BG: &str = "\x1b[48;2;40;44;52m";
const INLINE_CODE_LIGHT_BG: &str = "\x1b[48;2;230;233;238m";

const BLOCKQUOTE_DARK_BG: &str = QUOTE_BG_DARK;
const BLOCKQUOTE_LIGHT_BG: &str = QUOTE_BG_LIGHT;

const CODEBLOCK_DARK_FG: &str = DIM;
const CODEBLOCK_LIGHT_FG: &str = BRIGHT_BLACK;
const CODEBLOCK_DARK_BG: &str = CODE_BG_DARK;
const CODEBLOCK_LIGHT_BG: &str = CODE_BG_LIGHT;

const TABLE_DARK_FG: &str = CYAN;
const TABLE_LIGHT_FG: &str = BLACK;
const TABLE_HEADER_DARK_FG: &str = GREEN;
const TABLE_HEADER_LIGHT_FG: &str = BRIGHT_BLACK;

const TASK_CHECKED_DARK_FG: &str = GREEN;
const TASK_CHECKED_LIGHT_FG: &str = BLACK;
const TASK_UNCHECKED_DARK_FG: &str = YELLOW;
const TASK_UNCHECKED_LIGHT_FG: &str = BRIGHT_BLACK;

const IMAGE_DARK_FG: &str = MAGENTA;
const IMAGE_LIGHT_FG: &str = BLACK;

const FOOTNOTE_REF_DARK_FG: &str = YELLOW;
const FOOTNOTE_REF_LIGHT_FG: &str = BLUE;
const FOOTNOTE_BODY_DARK_FG: &str = GREEN;
const FOOTNOTE_BODY_LIGHT_FG: &str = BLACK;

#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
    pub width: usize,
    pub color: bool,
    pub theme: Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl RenderConfig {
    pub fn from_terminal(color: bool, theme: Theme) -> Self {
        let width = terminal_size()
            .map(|(Width(w), _)| usize::from(w))
            .filter(|w| *w > 0)
            .unwrap_or(80);

        Self { width, color, theme }
    }
}

#[derive(Debug, Clone)]
enum Block {
    Paragraph(Vec<InlineEvent>),
    Heading {
        level: u32,
        events: Vec<InlineEvent>,
    },
    CodeBlock {
        text: String,
    },
    FootnoteDefinition {
        label: String,
        blocks: Vec<Block>,
    },
    BlockQuote(Vec<Block>),
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
    Table {
        alignments: Vec<Alignment>,
        header: Vec<Vec<InlineEvent>>,
        rows: Vec<Vec<Vec<InlineEvent>>>,
    },
}

#[derive(Debug, Clone)]
struct ListItem {
    blocks: Vec<Block>,
    task: Option<bool>,
}

#[derive(Debug, Clone)]
enum InlineEvent {
    Text(String),
    Code(String),
    Image {
        alt: String,
        dest_url: String,
    },
    FootnoteRef(String),
    Start(InlineStyle),
    End(InlineStyle),
    SoftBreak,
    HardBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineStyle {
    Emphasis,
    Strong,
    Link,
    Code,
    Image,
    Footnote,
}

#[derive(Debug)]
enum Frame {
    Root {
        blocks: Vec<Block>,
    },
    BlockQuote {
        blocks: Vec<Block>,
    },
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
    Item {
        blocks: Vec<Block>,
        inline: Vec<InlineEvent>,
        task: Option<bool>,
    },
    Paragraph {
        events: Vec<InlineEvent>,
    },
    Heading {
        level: u32,
        events: Vec<InlineEvent>,
    },
    CodeBlock {
        text: String,
    },
    Image {
        dest_url: String,
        alt: Vec<InlineEvent>,
    },
    FootnoteDefinition {
        label: String,
        blocks: Vec<Block>,
    },
    Table {
        alignments: Vec<Alignment>,
        header: Vec<Vec<InlineEvent>>,
        rows: Vec<Vec<Vec<InlineEvent>>>,
        in_head: bool,
    },
    TableRow {
        cells: Vec<Vec<InlineEvent>>,
    },
    TableCell {
        events: Vec<InlineEvent>,
    },
}

pub fn render_file(path: impl AsRef<Path>, color: bool) -> io::Result<String> {
    let src = fs::read_to_string(path)?;
    Ok(render_markdown(&src, RenderConfig::from_terminal(color, Theme::Dark)))
}

pub fn render_file_with_theme(path: impl AsRef<Path>, color: bool, theme: Theme) -> io::Result<String> {
    let src = fs::read_to_string(path)?;
    Ok(render_markdown(&src, RenderConfig::from_terminal(color, theme)))
}

pub fn render_markdown(src: &str, config: RenderConfig) -> String {
    let blocks = parse_blocks(src);
    let mut out = String::new();

    for (idx, block) in blocks.iter().enumerate() {
        if idx > 0 {
            out.push_str("\n\n");
        }
        render_block(block, &mut out, config, "", 0);
    }

    out
}

fn parse_blocks(src: &str) -> Vec<Block> {
    let mut stack = vec![Frame::Root { blocks: Vec::new() }];
    let parser = Parser::new_ext(
        src,
        Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_FOOTNOTES,
    );

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => stack.push(Frame::Paragraph { events: Vec::new() }),
                Tag::Heading { level, .. } => stack.push(Frame::Heading {
                    level: heading_level_to_u32(level),
                    events: Vec::new(),
                }),
                Tag::BlockQuote(_) => stack.push(Frame::BlockQuote { blocks: Vec::new() }),
                Tag::List(start) => stack.push(Frame::List {
                    ordered: start.is_some(),
                    items: Vec::new(),
                }),
                Tag::Item => stack.push(Frame::Item {
                    blocks: Vec::new(),
                    inline: Vec::new(),
                    task: None,
                }),
                Tag::CodeBlock(_) => stack.push(Frame::CodeBlock {
                    text: String::new(),
                }),
                Tag::Image { dest_url, .. } => stack.push(Frame::Image {
                    dest_url: dest_url.into_string(),
                    alt: Vec::new(),
                }),
                Tag::FootnoteDefinition(label) => stack.push(Frame::FootnoteDefinition {
                    label: label.into_string(),
                    blocks: Vec::new(),
                }),
                Tag::Table(alignments) => stack.push(Frame::Table {
                    alignments,
                    header: Vec::new(),
                    rows: Vec::new(),
                    in_head: false,
                }),
                Tag::TableHead => {
                    if let Some(Frame::Table { in_head, .. }) = stack.last_mut() {
                        *in_head = true;
                    }
                }
                Tag::TableRow => stack.push(Frame::TableRow { cells: Vec::new() }),
                Tag::TableCell => stack.push(Frame::TableCell { events: Vec::new() }),
                Tag::Emphasis => push_inline_event(&mut stack, InlineEvent::Start(InlineStyle::Emphasis)),
                Tag::Strong => push_inline_event(&mut stack, InlineEvent::Start(InlineStyle::Strong)),
                Tag::Link { .. } => push_inline_event(&mut stack, InlineEvent::Start(InlineStyle::Link)),
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Paragraph => finish_paragraph(&mut stack),
                TagEnd::Heading(_) => finish_heading(&mut stack),
                TagEnd::BlockQuote(_) => finish_blockquote(&mut stack),
                TagEnd::List(_) => finish_list(&mut stack),
                TagEnd::Item => finish_item(&mut stack),
                TagEnd::CodeBlock => finish_codeblock(&mut stack),
                TagEnd::Image => finish_image(&mut stack),
                TagEnd::FootnoteDefinition => finish_footnote_definition(&mut stack),
                TagEnd::Table => finish_table(&mut stack),
                TagEnd::TableHead => {
                    if let Some(Frame::Table { in_head, .. }) = stack.last_mut() {
                        *in_head = false;
                    }
                }
                TagEnd::TableRow => finish_table_row(&mut stack),
                TagEnd::TableCell => finish_table_cell(&mut stack),
                TagEnd::Emphasis => push_inline_event(&mut stack, InlineEvent::End(InlineStyle::Emphasis)),
                TagEnd::Strong => push_inline_event(&mut stack, InlineEvent::End(InlineStyle::Strong)),
                TagEnd::Link => push_inline_event(&mut stack, InlineEvent::End(InlineStyle::Link)),
                _ => {}
            },
            Event::Text(text) => match stack.last_mut() {
                Some(Frame::Paragraph { events }) | Some(Frame::Heading { events, .. }) => {
                    events.push(InlineEvent::Text(text.into_string()))
                }
                Some(Frame::Item { inline, .. }) => inline.push(InlineEvent::Text(text.into_string())),
                Some(Frame::CodeBlock { text: code }) => code.push_str(&text),
                Some(Frame::Image { alt, .. }) => alt.push(InlineEvent::Text(text.into_string())),
                Some(Frame::FootnoteDefinition { .. }) => {
                    // Footnote definition bodies are handled via nested block frames.
                }
                Some(Frame::TableCell { events }) => events.push(InlineEvent::Text(text.into_string())),
                _ => {}
            },
            Event::Code(text) => push_inline_event(&mut stack, InlineEvent::Code(text.into_string())),
            Event::SoftBreak => push_inline_event(&mut stack, InlineEvent::SoftBreak),
            Event::HardBreak => push_inline_event(&mut stack, InlineEvent::HardBreak),
            Event::Rule => push_block(&mut stack, Block::CodeBlock { text: "------".to_string() }),
            Event::FootnoteReference(text) => push_inline_event(&mut stack, InlineEvent::FootnoteRef(text.into_string())),
            Event::TaskListMarker(checked) => {
                if let Some(Frame::Item { task, .. }) = stack.last_mut() {
                    *task = Some(checked);
                }
            }
            _ => {}
        }
    }

    match stack.pop() {
        Some(Frame::Root { blocks }) => blocks,
        _ => Vec::new(),
    }
}

fn push_inline_event(stack: &mut [Frame], event: InlineEvent) {
    match stack.last_mut() {
        Some(Frame::Paragraph { events }) | Some(Frame::Heading { events, .. }) => events.push(event),
        Some(Frame::Item { inline, .. }) => inline.push(event),
        Some(Frame::Image { alt, .. }) => alt.push(event),
        Some(Frame::TableCell { events }) => events.push(event),
        _ => {}
    }
}

fn push_block(stack: &mut Vec<Frame>, block: Block) {
    match stack.last_mut() {
        Some(Frame::Root { blocks })
        | Some(Frame::BlockQuote { blocks })
        | Some(Frame::Item { blocks, .. })
        | Some(Frame::FootnoteDefinition { blocks, .. }) => {
            blocks.push(block);
        }
        _ => {}
    }
}

fn finish_paragraph(stack: &mut Vec<Frame>) {
    if let Some(Frame::Paragraph { events }) = stack.pop() {
        push_block(stack, Block::Paragraph(events));
    }
}

fn finish_heading(stack: &mut Vec<Frame>) {
    if let Some(Frame::Heading { level, events }) = stack.pop() {
        push_block(stack, Block::Heading { level, events });
    }
}

fn finish_codeblock(stack: &mut Vec<Frame>) {
    if let Some(Frame::CodeBlock { text }) = stack.pop() {
        push_block(stack, Block::CodeBlock { text });
    }
}

fn finish_blockquote(stack: &mut Vec<Frame>) {
    if let Some(Frame::BlockQuote { blocks }) = stack.pop() {
        push_block(stack, Block::BlockQuote(blocks));
    }
}

fn finish_footnote_definition(stack: &mut Vec<Frame>) {
    if let Some(Frame::FootnoteDefinition { label, blocks }) = stack.pop() {
        push_block(stack, Block::FootnoteDefinition { label, blocks });
    }
}

fn finish_item(stack: &mut Vec<Frame>) {
    if let Some(Frame::Item { mut blocks, inline, task }) = stack.pop() {
        if !inline.is_empty() {
            blocks.push(Block::Paragraph(inline));
        }
        if let Some(Frame::List { items, .. }) = stack.last_mut() {
            items.push(ListItem { blocks, task });
        }
    }
}

fn finish_list(stack: &mut Vec<Frame>) {
    if let Some(Frame::List { ordered, items }) = stack.pop() {
        push_block(stack, Block::List { ordered, items });
    }
}

fn finish_image(stack: &mut Vec<Frame>) {
    if let Some(Frame::Image { dest_url, alt }) = stack.pop() {
        let alt_text = render_inline_plain(&alt);
        push_inline_event(stack, InlineEvent::Image { alt: alt_text, dest_url });
    }
}

fn finish_table(stack: &mut Vec<Frame>) {
    if let Some(Frame::Table {
        alignments,
        header,
        rows,
        ..
    }) = stack.pop()
    {
        push_block(stack, Block::Table {
            alignments,
            header,
            rows,
        });
    }
}

fn finish_table_row(stack: &mut Vec<Frame>) {
    if let Some(Frame::TableRow { cells }) = stack.pop() {
        if let Some(Frame::Table { header, rows, in_head, .. }) = stack.last_mut() {
            if *in_head || (header.is_empty() && rows.is_empty()) {
                *header = cells;
            } else {
                rows.push(cells);
            }
        }
    }
}

fn finish_table_cell(stack: &mut Vec<Frame>) {
    if let Some(Frame::TableCell { events }) = stack.pop() {
        if let Some(Frame::TableRow { cells }) = stack.last_mut() {
            cells.push(events);
        }
    }
}

fn heading_level_to_u32(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn render_block(block: &Block, out: &mut String, config: RenderConfig, prefix: &str, depth: usize) {
    match block {
        Block::Paragraph(events) => {
            out.push_str(&render_inline(events, config, prefix, ""));
        }
        Block::Heading { level, events } => {
            out.push_str(&render_heading(events, *level, config, prefix));
        }
        Block::CodeBlock { text } => {
            out.push_str(&render_code_block(text, config, prefix));
        }
        Block::FootnoteDefinition { label, blocks } => {
            out.push_str(&render_footnote_definition(label, blocks, config, prefix));
        }
        Block::BlockQuote(blocks) => {
            let quote_prefix = format!("{prefix}> ");
            let mut inner = String::new();
            for (idx, child) in blocks.iter().enumerate() {
                if idx > 0 {
                    inner.push('\n');
                }
                render_block(child, &mut inner, config, &quote_prefix, depth + 1);
            }
            if config.color {
                out.push_str(&style_block_background(&inner, quote_bg(config.theme), config.width));
            } else {
                out.push_str(&inner);
            }
        }
        Block::List { ordered, items } => {
            for (idx, item) in items.iter().enumerate() {
                let marker = if *ordered {
                    format!("{}. ", idx + 1)
                } else {
                    "- ".to_string()
                };
                if idx > 0 {
                    out.push('\n');
                }
                render_list_item(item, out, config, prefix, &marker, depth);
            }
        }
        Block::Table {
            alignments,
            header,
            rows,
        } => {
            out.push_str(&render_table(alignments, header, rows, config, prefix));
        }
    }
}

fn render_list_item(
    item: &ListItem,
    out: &mut String,
    config: RenderConfig,
    prefix: &str,
    marker: &str,
    depth: usize,
) {
    let marker_text = render_list_marker(marker, item.task, config);
    let marker_width = if item.task.is_some() {
        visible_width("[x] ")
    } else {
        visible_width(marker)
    };
    let first_prefix = format!("{prefix}{marker_text}");
    let continuation_prefix = format!("{prefix}{}", " ".repeat(marker_width));

    for (idx, block) in item.blocks.iter().enumerate() {
        if idx > 0 {
            out.push_str("\n\n");
        }
        match block {
            Block::Paragraph(events) => {
                out.push_str(&render_inline(events, config, &first_prefix, &continuation_prefix));
            }
            Block::Heading { level, events } => {
                out.push_str(&render_heading(events, *level, config, &first_prefix));
            }
            Block::CodeBlock { text } => {
                out.push_str(&render_code_block(text, config, &continuation_prefix));
            }
            Block::FootnoteDefinition { label, blocks } => {
                out.push_str(&render_footnote_definition(label, blocks, config, &continuation_prefix));
            }
            Block::BlockQuote(children) => {
                let quote_prefix = format!("{prefix}{marker_text}> ");
                let mut inner = String::new();
                for (child_idx, child) in children.iter().enumerate() {
                    if child_idx > 0 {
                        inner.push_str("\n\n");
                    }
                    render_block(child, &mut inner, config, &quote_prefix, depth + 1);
                }
                out.push_str(&style_block_background(&inner, quote_bg(config.theme), config.width));
            }
            Block::List { .. } => {
                render_block(block, out, config, &continuation_prefix, depth + 1);
            }
            Block::Table {
                alignments,
                header,
                rows,
            } => {
                out.push_str(&render_table(alignments, header, rows, config, &continuation_prefix));
            }
        }
    }
}

fn render_heading(events: &[InlineEvent], level: u32, config: RenderConfig, prefix: &str) -> String {
    let marker = "#".repeat(level as usize);
    let body = render_inline(events, config, "", "");
    let (fg, bg, underline) = match (config.theme, level) {
        (Theme::Dark, 1) => (HEADING_1_DARK_FG, HEADING_1_DARK_BG, false),
        (Theme::Dark, 2) => (HEADING_2_DARK_FG, "", true),
        (Theme::Dark, 3) => (HEADING_3_DARK_FG, "", false),
        (Theme::Dark, 4) => (HEADING_4_DARK_FG, "", false),
        (Theme::Dark, 5) => (HEADING_5_DARK_FG, "", false),
        (Theme::Dark, _) => (HEADING_6_DARK_FG, "", false),
        (Theme::Light, 1) => (HEADING_1_LIGHT_FG, HEADING_1_LIGHT_BG, false),
        (Theme::Light, 2) => (HEADING_2_LIGHT_FG, "", true),
        (Theme::Light, 3) => (HEADING_3_LIGHT_FG, "", false),
        (Theme::Light, 4) => (HEADING_4_LIGHT_FG, "", false),
        (Theme::Light, 5) => (HEADING_5_LIGHT_FG, "", false),
        (Theme::Light, _) => (HEADING_6_LIGHT_FG, "", false),
    };
    let mut style = String::new();
    style.push_str(BOLD);
    if underline {
        style.push_str(UNDERLINE);
    }
    style.push_str(fg);
    if !bg.is_empty() {
        style.push_str(bg);
    }

    let mut text = format!("{prefix}{marker} {body}");
    if config.color {
        text = format!("{style}{text}{RESET}");
    }

    text
}

fn render_table(
    alignments: &[Alignment],
    header: &[Vec<InlineEvent>],
    rows: &[Vec<Vec<InlineEvent>>],
    config: RenderConfig,
    prefix: &str,
) -> String {
    let header_cells: Vec<String> = header.iter().map(|cell| render_inline_plain(cell)).collect();
    let body_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| row.iter().map(|cell| render_inline_plain(cell)).collect())
        .collect();

    let column_count = header_cells
        .len()
        .max(body_rows.iter().map(|row| row.len()).max().unwrap_or(0));
    let mut widths = vec![0usize; column_count];

    for (idx, cell) in header_cells.iter().enumerate() {
        widths[idx] = widths[idx].max(visible_width(cell));
    }
    for row in &body_rows {
        for (idx, cell) in row.iter().enumerate() {
            widths[idx] = widths[idx].max(visible_width(cell));
        }
    }

    let mut out = String::new();
    let table_color = if config.color { table_text_color(config.theme) } else { "" };
    let header_color = if config.color { table_header_color(config.theme) } else { "" };

    if !header_cells.is_empty() {
        out.push_str(&format_table_row(
            prefix,
            &header_cells,
            &widths,
            table_color,
            Some((header_color, true)),
        ));
        out.push('\n');
        out.push_str(prefix);
        if config.color {
            out.push_str(table_color);
        }
        out.push('|');
        out.push(' ');
        for col_idx in 0..column_count {
            let sep = table_separator(widths[col_idx], alignments.get(col_idx).copied().unwrap_or(Alignment::None));
            out.push_str(&sep);
            if col_idx + 1 < column_count {
                out.push_str(" | ");
            } else {
                out.push(' ');
            }
        }
        out.push('|');
        if config.color {
            out.push_str(RESET);
        }
    }

    for row in &body_rows {
        out.push('\n');
        out.push_str(&format_table_row(prefix, row, &widths, table_color, None));
    }

    out
}

fn render_code_block(text: &str, config: RenderConfig, prefix: &str) -> String {
    let mut raw = String::new();
    for line in text.lines() {
        let wrapped = wrap_raw_line(line, config.width, prefix);
        for (idx, wrapped_line) in wrapped.iter().enumerate() {
            if idx > 0 {
                raw.push('\n');
            }
            if config.color {
                raw.push_str(code_block_prefix(config.theme));
                raw.push_str(wrapped_line);
                raw.push_str(RESET);
            } else {
                raw.push_str(wrapped_line);
            }
        }
        raw.push('\n');
    }
    while raw.ends_with('\n') {
        raw.pop();
    }
    if config.color {
        style_block_background(&raw, code_bg(config.theme), config.width)
    } else {
        raw
    }
}

fn render_footnote_definition(label: &str, blocks: &[Block], config: RenderConfig, prefix: &str) -> String {
    let marker = format!("[^{}]: ", label);
    let continuation_prefix = format!("{prefix}{}", " ".repeat(visible_width(&marker)));
    let mut out = String::new();
    let marker_style = footnote_color(config.theme);
    let body_style = footnote_body_color(config.theme);

    if config.color {
        out.push_str(prefix);
        out.push_str(marker_style);
        out.push_str(&marker);
        out.push_str(RESET);
    } else {
        out.push_str(prefix);
        out.push_str(&marker);
    }

    for (idx, block) in blocks.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        let body = match block {
            Block::Paragraph(events) => render_inline(events, config, "", &continuation_prefix),
            _ => render_block_to_string(block, config, &continuation_prefix, 0),
        };
        match block {
            Block::Paragraph(_) | Block::Heading { .. } | Block::CodeBlock { .. } | Block::BlockQuote(_) | Block::List { .. } | Block::Table { .. } | Block::FootnoteDefinition { .. } => {
                if config.color {
                    out.push_str(body_style);
                    out.push_str(&body);
                    out.push_str(RESET);
                } else {
                    out.push_str(&body);
                }
            }
        }
    }

    out
}

fn render_block_to_string(block: &Block, config: RenderConfig, prefix: &str, depth: usize) -> String {
    let mut out = String::new();
    render_block(block, &mut out, config, prefix, depth);
    out
}

fn format_table_row(
    prefix: &str,
    cells: &[String],
    widths: &[usize],
    table_color: &str,
    header: Option<(&str, bool)>,
) -> String {
    let mut out = String::new();
    out.push_str(prefix);
    if !table_color.is_empty() {
        out.push_str(table_color);
    }
    out.push('|');
    out.push(' ');

    for col_idx in 0..widths.len() {
        let cell = cells.get(col_idx).cloned().unwrap_or_default();
        let cell_width = visible_width(&cell);

        if let Some((header_color, bold)) = header {
            if !header_color.is_empty() {
                out.push_str(header_color);
            }
            if bold {
                out.push_str(BOLD);
            }
            out.push_str(&cell);
            if !table_color.is_empty() {
                out.push_str(RESET);
                out.push_str(table_color);
            }
        } else {
            out.push_str(&cell);
        }

        let pad = widths[col_idx].saturating_sub(cell_width);
        out.push_str(&" ".repeat(pad));
        if col_idx + 1 < widths.len() {
            out.push_str(" | ");
        } else {
            out.push(' ');
        }
    }

    out.push('|');
    if !table_color.is_empty() {
        out.push_str(RESET);
    }
    out
}

fn table_separator(width: usize, alignment: Alignment) -> String {
    let _ = alignment;
    "-".repeat(width.max(3))
}

fn render_inline(events: &[InlineEvent], config: RenderConfig, first_prefix: &str, continuation_prefix: &str) -> String {
    let spans = inline_spans(events);
    wrap_spans(&spans, config.width, first_prefix, continuation_prefix, config.color, config.theme)
}

fn inline_spans(events: &[InlineEvent]) -> Vec<(String, Vec<InlineStyle>)> {
    let mut spans = Vec::new();
    let mut style_stack: Vec<InlineStyle> = Vec::new();

    for event in events {
        match event {
            InlineEvent::Text(text) => spans.push((normalize_text(text), style_stack.clone())),
            InlineEvent::Code(text) => spans.push((text.clone(), vec![InlineStyle::Code])),
            InlineEvent::Image { alt, dest_url } => {
                spans.push((format!("![{}]({})", alt, dest_url), vec![InlineStyle::Image]))
            }
            InlineEvent::FootnoteRef(label) => {
                spans.push((format!("[^{}]", label), vec![InlineStyle::Footnote]))
            }
            InlineEvent::Start(style) => style_stack.push(*style),
            InlineEvent::End(style) => {
                if let Some(pos) = style_stack.iter().rposition(|s| s == style) {
                    style_stack.remove(pos);
                }
            }
            InlineEvent::SoftBreak => spans.push((" ".to_string(), style_stack.clone())),
            InlineEvent::HardBreak => spans.push(("\n".to_string(), style_stack.clone())),
        }
    }

    spans
}

fn wrap_spans(
    spans: &[(String, Vec<InlineStyle>)],
    width: usize,
    first_prefix: &str,
    continuation_prefix: &str,
    color: bool,
    theme: Theme,
) -> String {
    let mut out = String::new();
    let mut line = String::new();
    let mut line_width = 0usize;
    let mut first_line = true;
    let mut prefix = first_prefix.to_string();

    let tokens = tokenize_spans(spans);

    for token in tokens {
        match token {
            Token::Break => {
                flush_line(&mut out, &mut line, &mut line_width, &mut first_line, &mut prefix, first_prefix, continuation_prefix);
            }
            Token::Word { text, styles } => {
                let token_width = visible_width(&text);
                let prefix_width = visible_width(&prefix);
                let available = width.saturating_sub(prefix_width);
                if line_width > 0 && line_width + 1 + token_width > available {
                    flush_line(
                        &mut out,
                        &mut line,
                        &mut line_width,
                        &mut first_line,
                        &mut prefix,
                        first_prefix,
                        continuation_prefix,
                    );
                }

                if line_width > 0 && !is_punctuation_token(&text) {
                    line.push(' ');
                    line_width += 1;
                }

                line.push_str(&styled_text(&text, &styles, color, theme));
                line_width += token_width;
            }
        }
    }

    flush_line(
        &mut out,
        &mut line,
        &mut line_width,
        &mut first_line,
        &mut prefix,
        first_prefix,
        continuation_prefix,
    );

    out
}

fn is_punctuation_token(token: &str) -> bool {
    !token.is_empty() && token.chars().all(|ch| matches!(ch, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}'))
}

enum Token {
    Word { text: String, styles: Vec<InlineStyle> },
    Break,
}

fn tokenize_spans(spans: &[(String, Vec<InlineStyle>)]) -> Vec<Token> {
    let mut tokens = Vec::new();
    for (text, styles) in spans {
        if text == "\n" {
            tokens.push(Token::Break);
            continue;
        }

        if styles
            .iter()
            .any(|s| matches!(s, InlineStyle::Code | InlineStyle::Image | InlineStyle::Footnote))
        {
            tokens.push(Token::Word {
                text: text.clone(),
                styles: styles.clone(),
            });
            continue;
        }

        for word in text.split_whitespace() {
            tokens.push(Token::Word {
                text: word.to_string(),
                styles: styles.clone(),
            });
        }
    }

    tokens
}

fn flush_line(
    out: &mut String,
    line: &mut String,
    line_width: &mut usize,
    first_line: &mut bool,
    prefix: &mut String,
    first_prefix: &str,
    continuation_prefix: &str,
) {
    if !line.is_empty() || *first_line {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(prefix);
        out.push_str(line);
    }
    line.clear();
    *line_width = 0;
    *first_line = false;
    *prefix = continuation_prefix.to_string();
    if prefix.is_empty() {
        *prefix = first_prefix.to_string();
    }
}

fn styled_text(text: &str, styles: &[InlineStyle], color: bool, theme: Theme) -> String {
    if !color || styles.is_empty() {
        return text.to_string();
    }

    let mut prefix = String::new();
    for style in styles {
        match style {
            InlineStyle::Emphasis => prefix.push_str(ITALIC),
            InlineStyle::Strong => prefix.push_str(BOLD),
            InlineStyle::Link => {
                prefix.push_str(link_color(theme));
                prefix.push_str(UNDERLINE);
            }
            InlineStyle::Code => {
                prefix.push_str(code_inline_fg(theme));
                prefix.push_str(code_inline_bg(theme));
            }
            InlineStyle::Image => prefix.push_str(image_color(theme)),
            InlineStyle::Footnote => prefix.push_str(footnote_color(theme)),
        }
    }

    format!("{prefix}{text}{RESET}")
}

fn link_color(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => LINK_DARK_FG,
        Theme::Light => LINK_LIGHT_FG,
    }
}

fn code_inline_fg(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => INLINE_CODE_DARK_FG,
        Theme::Light => INLINE_CODE_LIGHT_FG,
    }
}

fn code_inline_bg(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => INLINE_CODE_DARK_BG,
        Theme::Light => INLINE_CODE_LIGHT_BG,
    }
}

fn code_block_prefix(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => CODEBLOCK_DARK_FG,
        Theme::Light => CODEBLOCK_LIGHT_FG,
    }
}

fn quote_bg(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => BLOCKQUOTE_DARK_BG,
        Theme::Light => BLOCKQUOTE_LIGHT_BG,
    }
}

fn code_bg(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => CODEBLOCK_DARK_BG,
        Theme::Light => CODEBLOCK_LIGHT_BG,
    }
}

fn table_text_color(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => TABLE_DARK_FG,
        Theme::Light => TABLE_LIGHT_FG,
    }
}

fn table_header_color(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => TABLE_HEADER_DARK_FG,
        Theme::Light => TABLE_HEADER_LIGHT_FG,
    }
}

fn task_checked_color(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => TASK_CHECKED_DARK_FG,
        Theme::Light => TASK_CHECKED_LIGHT_FG,
    }
}

fn task_unchecked_color(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => TASK_UNCHECKED_DARK_FG,
        Theme::Light => TASK_UNCHECKED_LIGHT_FG,
    }
}

fn style_block_background(text: &str, bg: &str, width: usize) -> String {
    let mut out = String::new();
    let trimmed = text.trim_end_matches('\n');
    for (idx, line) in trimmed.split('\n').enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        let body = line.replace(RESET, &format!("{RESET}{bg}"));
        out.push_str(bg);
        out.push_str(&body);
        let pad = width.saturating_sub(visible_width(line));
        out.push_str(&" ".repeat(pad));
        out.push_str(RESET);
    }

    out
}

fn render_inline_plain(events: &[InlineEvent]) -> String {
    let mut out = String::new();

    for event in events {
        match event {
            InlineEvent::Text(text) => {
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push_str(&normalize_text(text));
            }
            InlineEvent::Code(text) => {
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push_str(text);
            }
            InlineEvent::Image { alt, dest_url } => {
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push_str(&format!("![{}]({})", alt, dest_url));
            }
            InlineEvent::FootnoteRef(label) => {
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push_str(&format!("[^{}]", label));
            }
            InlineEvent::SoftBreak | InlineEvent::HardBreak => {
                if !out.ends_with(' ') && !out.is_empty() {
                    out.push(' ');
                }
            }
            InlineEvent::Start(_) | InlineEvent::End(_) => {}
        }
    }

    out.trim().to_string()
}

fn render_list_marker(marker: &str, task: Option<bool>, config: RenderConfig) -> String {
    if let Some(checked) = task {
        let plain = if checked { "[x] " } else { "[ ] " };
        let color = if checked {
            task_checked_color(config.theme)
        } else {
            task_unchecked_color(config.theme)
        };
        if config.color {
            return format!("{color}{plain}{RESET}");
        }
        return plain.to_string();
    }

    marker.to_string()
}

fn image_color(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => IMAGE_DARK_FG,
        Theme::Light => IMAGE_LIGHT_FG,
    }
}

fn footnote_color(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => FOOTNOTE_REF_DARK_FG,
        Theme::Light => FOOTNOTE_REF_LIGHT_FG,
    }
}

fn footnote_body_color(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => FOOTNOTE_BODY_DARK_FG,
        Theme::Light => FOOTNOTE_BODY_LIGHT_FG,
    }
}

fn wrap_raw_line(line: &str, width: usize, prefix: &str) -> Vec<String> {
    let available = width.saturating_sub(visible_width(prefix));
    if available == 0 {
        return vec![format!("{prefix}{line}")];
    }

    let mut result = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;

    for ch in line.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if current_width > 0 && current_width + ch_width > available {
            result.push(format!("{prefix}{current}"));
            current.clear();
            current_width = 0;
        }
        current.push(ch);
        current_width += ch_width;
    }

    result.push(format!("{prefix}{current}"));
    result
}

fn visible_width(text: &str) -> usize {
    strip_ansi(text)
        .chars()
        .map(|ch| ch.width().unwrap_or(0))
        .sum()
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            while let Some(next) = chars.next() {
                if next == 'm' {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn normalize_text(text: &str) -> String {
    let mut out = String::new();
    let mut last_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            out.push(ch);
            last_space = false;
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_heading_and_paragraph() {
        let src = "# Title\n\nHello **world**.";
        let rendered = render_markdown(
            src,
            RenderConfig {
                width: 80,
                color: false,
                theme: Theme::Dark,
            },
        );

        assert!(rendered.contains("# Title"));
        assert!(rendered.contains("Hello world."));
    }

    #[test]
    fn wraps_paragraphs() {
        let src = "This is a very long paragraph that should wrap to the configured terminal width without using terminal control sequences.";
        let rendered = render_markdown(
            src,
            RenderConfig {
                width: 40,
                color: false,
                theme: Theme::Dark,
            },
        );

        assert!(rendered.lines().any(|line| line.len() <= 40));
    }

    #[test]
    fn renders_tight_lists() {
        let src = "* one\n* two";
        let rendered = render_markdown(
            src,
            RenderConfig {
                width: 80,
                color: false,
                theme: Theme::Dark,
            },
        );

        assert!(rendered.contains("- one") || rendered.contains("* one"));
        assert!(rendered.contains("- two") || rendered.contains("* two"));
    }

    #[test]
    fn renders_task_lists_and_images() {
        let src = "- [x] done\n- [ ] todo\n\n![alt text](https://example.com/image.png)";
        let rendered = render_markdown(
            src,
            RenderConfig {
                width: 80,
                color: false,
                theme: Theme::Dark,
            },
        );

        assert!(rendered.contains("[x] done"));
        assert!(rendered.contains("[ ] todo"));
        assert!(rendered.contains("![alt text](https://example.com/image.png)"));
    }

    #[test]
    fn renders_pipe_tables() {
        let src = "| A | B |\n| --- | --- |\n| one | two |";
        let rendered = render_markdown(
            src,
            RenderConfig {
                width: 80,
                color: false,
                theme: Theme::Dark,
            },
        );

        assert!(rendered.contains("|"));
        assert!(rendered.contains("one"));
        assert!(rendered.contains("two"));
    }

    #[test]
    fn renders_footnotes() {
        let src = "Footnote here.[^1]\n\n[^1]: Footnote body.";
        let rendered = render_markdown(
            src,
            RenderConfig {
                width: 80,
                color: false,
                theme: Theme::Dark,
            },
        );

        assert!(rendered.contains("[^1]"));
        assert!(rendered.contains("[^1]: Footnote body."));
    }
}
