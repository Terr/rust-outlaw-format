use std::cmp::Ordering;

use crate::{Block, Document, FormattedLine, LineType, RawLine};

/// Parses the lines of `contents` and determines the type of line (header, bullet point list,
/// etc.) and decides the indenting each line needs to get.
pub fn parse_document(contents: &str) -> Document {
    let mut document = Document::new();

    for line in contents.lines() {
        let raw_line = RawLine::from_string(line);

        if raw_line.is_header() {
            // Finding a header means the start of a new Block

            let indent_level = determine_new_header_indent(&document, &raw_line);
            let header = FormattedLine::from_raw(raw_line, indent_level);

            document.add_block(Block::new(header));
        } else if raw_line.is_bullet_point() {
            // This case means that the line is either the start of a new bullet point list, or the
            // continuation of one.

            let current_block = document.last_block_mut();
            let indent_level = determine_new_bullet_point_indent(current_block, &raw_line);
            let bullet_point_line = FormattedLine::from_raw(raw_line, indent_level);

            current_block.add_line(bullet_point_line);
        } else {
            // In this case the line is either a normal line of text, some prefixed line (like a
            // quote or preformatted) or the continuation of a (line wrapped) bullet point.

            let current_block = document.last_block_mut();
            let line = parse_text_line(current_block, raw_line);

            current_block.add_line(line);
        };
    }

    document
}

/// Determines if the given line is a child, sibling or parent of the previous block's header
fn determine_new_header_indent(document: &Document, raw_line: &RawLine) -> usize {
    assert!(raw_line.is_header());

    let previous_block = document.last_block();

    match previous_block.raw_header_indent().cmp(&raw_line.num_indent) {
        // New header is a sibling (at the same level) of the previous header
        Ordering::Equal => previous_block.indent_level(),

        // New header is a parent of *a* previous header
        Ordering::Greater => document
            .find_latest_block_with_raw_indent(raw_line.num_indent)
            .map(|block| block.indent_level())
            .unwrap_or(0),

        // New header is a child of the previous header
        Ordering::Less => previous_block.indent_level() + 1,
    }
}

fn determine_new_bullet_point_indent(current_block: &Block, raw_line: &RawLine) -> usize {
    assert!(raw_line.is_bullet_point());

    if let Some(previous_bullet_point) = current_block.find_previous_of(LineType::ListBulletPoint) {
        match previous_bullet_point
            .original_raw
            .num_indent
            .cmp(&raw_line.num_indent)
        {
            // List item is continuation of the bullet point list at the same level of
            // indenting.
            Ordering::Equal => previous_bullet_point.indent_level,

            // List item is shifted one or more levels to the left compared to the previous
            // bullet point in the list. Find the first line (starting from the last line)
            // that had the same indenting in the original raw file.
            Ordering::Greater => current_block
                .find_latest_line_with_raw_indent(raw_line.num_indent)
                .map(|line| line.indent_level)
                .unwrap_or(current_block.indent_level() + 1),

            // List item is shifted right compared to the previous bullet point. Only one
            // level of indenting per line can be added per line.
            Ordering::Less => previous_bullet_point.indent_level + 1,
        }
    } else if let Some(previous_text) = current_block.find_previous_of(LineType::Text) {
        previous_text.indent_level
    } else {
        0
    }
}

fn parse_text_line(current_block: &mut Block, raw_line: RawLine) -> FormattedLine {
    if let Some(previous_line) = current_block.last_line() {
        if previous_line.is_list_item() && !raw_line.is_empty() {
            FormattedLine {
                indent_level: previous_line.indent_level,
                line_type: LineType::ListContinuousLine,
                contents: format!(
                    "{}{}",
                    LineType::ListContinuousLine.get_prefix(),
                    &raw_line.trimmed
                ),
                original_raw: raw_line,
            }
        } else if current_block.has_header() {
            // Non-bullet list Contents of a block follow the block's indent level plus one
            FormattedLine::from_raw(raw_line, current_block.indent_level() + 1)
        } else {
            // This applies to empty lines and to lines of text that are placed before the
            // first header line.

            FormattedLine::from_raw(raw_line, current_block.indent_level())
        }
    } else {
        // This applies to the first line after a header.

        FormattedLine::from_raw(raw_line, current_block.indent_level() + 1)
    }
}
