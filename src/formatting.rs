use crate::{Document, FormattedLine, LineType};

#[derive(Debug, Eq, PartialEq)]
enum Action {
    Start,
    InsertBlankLine,
    InsertBodyText,
}

/// Splits lines longer than `max_line_length` and inserts a newline at the nearest space preceding
/// that point in the line.
///
/// Because this function modifies the original, already formatted line without any regard for the
/// indenting the added line needs, the return value indicates if the document needs reformatting.
pub fn wrap_long_lines(formatted_lines: &mut Vec<FormattedLine>, max_line_length: usize) {
    // This is a for loop instead of a 'real' loop to guard against any infinite loops. The '100'
    // is chosen arbitrarily
    let max_iterations = 100;
    let mut lines_were_changed = None;
    for iteration in 0..max_iterations {
        // See if there are (still) lines that need to be wrapped because they are too long

        // If the previous iteration didn't modify any lines it means that there aren't any lines
        // left to wrap, so we can stop
        if let Some(false) = lines_were_changed {
            break;
        }

        if formatted_lines.iter().all(|line| {
            line.line_type == LineType::Preformatted || line.contents.len() <= max_line_length
        }) {
            // Nothing more to be done
            break;
        }

        let mut lines_to_insert: Vec<(usize, FormattedLine)> = Vec::new();

        for (index, current_line) in formatted_lines.iter_mut().enumerate() {
            if current_line.line_type == LineType::Preformatted {
                continue;
            }

            if current_line.contents.len() <= max_line_length {
                continue;
            }

            // Find a word boundary to split the string at
            let Some(split_pos) = find_word_boundary(current_line, max_line_length) else { continue; };

            // This FormattedLine will be placed below (line index + 1) the `current_line` in
            // the document
            lines_to_insert.push((index + 1, split_line(current_line, split_pos)));
        }

        lines_were_changed = Some(!lines_to_insert.is_empty());
        formatted_lines.reserve(lines_to_insert.len());
        for (index_offset, (index, line_to_insert)) in lines_to_insert.into_iter().enumerate() {
            // For every line that is added (inserted) it increases the line number for the next
            // to-be-inserted line by 1. `index_offset` keeps track of how much the following line
            // needs to be shifted downwards.
            formatted_lines.insert(index + index_offset, line_to_insert);
        }

        if iteration == max_iterations - 1 {
            eprintln!("wrap_long_lines() was possibly stuck in an infinite loop!");
        }
    }
}

pub fn format_to_string(document: &Document) -> String {
    let mut formatted = String::new();
    let mut last_action = Action::Start;

    for block in document.blocks.iter() {
        if last_action == Action::InsertBodyText {
            formatted += "\n";
        }

        formatted += &format!(
            "{indenting}{header}\n\n",
            indenting = " ".repeat(block.header.num_indent()),
            header = block.header.contents
        );
        last_action = Action::InsertBlankLine;

        for formatted_line in block.contents.iter() {
            if formatted_line.line_type != LineType::Preformatted
                && last_action == Action::InsertBlankLine
                && formatted_line.is_empty()
            {
                // Don't output multiple blank lines in a row for non-preformatted lines
                continue;
            }

            last_action = if formatted_line.is_empty() {
                formatted += "\n";

                Action::InsertBlankLine
            } else {
                formatted += &format!(
                    "{indenting}{line}\n",
                    indenting = " ".repeat(formatted_line.num_indent()),
                    line = formatted_line.contents
                );
                Action::InsertBodyText
            };
        }
    }

    formatted.trim_start().trim_end_matches(' ').to_owned()
}

/// Finds a word boundary (i.e. whitespace after a word) nearest to the maximum line length.
fn find_word_boundary(line: &FormattedLine, max_line_length: usize) -> Option<usize> {
    let prefix_length = line.line_type.get_prefix_length();
    if let Some(split_pos) = line
        .contents
        .chars()
        // We skip the prefix so that any whitespace in it will not satisfy the
        // `rfind()` below
        .skip(prefix_length)
        .take(max_line_length - prefix_length + 1)
        // String allocation is unfortunatly necessary here in order to use `rfind()`
        // because `Take` and `Skip` don't implement the necessary traits for it
        .collect::<String>()
        .rfind(|i: char| i.is_whitespace())
    {
        Some(prefix_length + split_pos)
    } else {
        // Line is too long but has no word boundary to split at within the first `max_line_length`
        // characters. This can happen if the line begins with a very long URL. Find the first word
        // boundary *after* the `max_line_length` (if any) in such cases.
        line.contents
            .chars()
            .skip(max_line_length)
            .position(|i| i.is_whitespace())
            .map(|split_pos| max_line_length + split_pos)
    }
}

/// Split a line at the specified position, modifying the original line and returning a new
/// `FormattedLine` with the contents after the split position.
fn split_line(long_line: &mut FormattedLine, split_pos: usize) -> FormattedLine {
    let (line_a, line_b) = long_line.contents.split_at(split_pos);

    let line_type = if long_line.is_list_item() {
        LineType::ListContinuousLine
    } else {
        long_line.line_type
    };

    let split_line = FormattedLine {
        contents: format!("{}{}", line_type.get_prefix(), line_b.trim()),
        line_type,

        ..long_line.clone()
    };

    long_line.contents = line_a.to_owned();

    split_line
}
