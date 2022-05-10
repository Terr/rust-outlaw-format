use std::convert::TryFrom;

use std::io::{self, Read};

static INDENT_SHIFT: i8 = 4;

#[derive(Debug, Eq, PartialEq)]
enum Action {
    Start,
    InsertHeader,
    InsertBlankLine,
    InsertBodyText,
}

#[derive(Debug, Eq, PartialEq)]
enum Context {
    Text,
    ListItem, // Currently processing a list item (started with a '* ')
}

fn main() {
    let mut stdin = io::stdin();
    let mut contents = String::new();
    stdin.read_to_string(&mut contents).expect("read stdin");

    //let input = File::open(path)?;
    //let buf_reader = io::BufReader::new(input);

    print!("{}", format(&contents));
}

fn format(contents: &str) -> String {
    let mut formatted = String::with_capacity(contents.len());

    let mut indent_level: i8 = -1;
    let mut context = Context::Text;
    let mut last_action = Action::Start;

    for line in contents.lines() {
        let trimmed_line = line.trim_start();
        let num_line_whitespace: i8 = (line.len() - trimmed_line.len()) as i8;
        let trimmed_line = trimmed_line.trim_end();

        // Write an empty line after a new header
        if last_action == Action::InsertHeader {
            formatted += "\n";
            last_action = Action::InsertBlankLine;

            if trimmed_line.len() == 0 {
                continue;
            }
        }

        if trimmed_line.len() == 0 {
            // Don't allow repeated blank lines
            if last_action != Action::InsertBlankLine {
                formatted += "\n";
                last_action = Action::InsertBlankLine;
            }

            // A blank line also means that if we were handling a List, we've now reached the end
            // of it
            context = Context::Text;

            continue;
        }

        let new_indent_width;
        if trimmed_line.starts_with("===") {
            // Make sure header follows after a blank line, unless it's the first line of the file
            #[allow(unused_assignments)]
            if last_action != Action::InsertBlankLine && last_action != Action::Start {
                formatted += "\n";
                last_action = Action::InsertBlankLine;
            }

            // A header means a new indent level if it is a child, parent or even grand parent of a
            // previous header. Calculate that indent level.
            if num_line_whitespace != indent_level * INDENT_SHIFT {
                if num_line_whitespace > (indent_level + 1) * INDENT_SHIFT {
                    indent_level += 1;
                } else {
                    indent_level = num_line_whitespace / INDENT_SHIFT;
                }
            }

            new_indent_width = indent_level * INDENT_SHIFT;
            last_action = Action::InsertHeader;
        } else {
            let mut body_text_ident: i8 = indent_level + 1;
            let mut extra_spaces: i8 = 0;

            // Lists are allowed to have extra indentation
            if trimmed_line.starts_with('*') {
                context = Context::ListItem;

                if num_line_whitespace > body_text_ident * INDENT_SHIFT {
                    // Assumes that the type of indentation (spaces or tabs) used for this line was
                    // the same as what was used for the header
                    body_text_ident = ((num_line_whitespace as f64
                        / (indent_level * INDENT_SHIFT) as f64)
                        .ceil()) as i8;
                }
            } else if context == Context::ListItem {
                // List items that have been broken up with newlines get some extra indenting, so
                // that the text of subsequent lines line up with the '* ' of the start of the list
                // item.
                extra_spaces += 2;

                let list_item_whitespace =
                    if num_line_whitespace % 4 != 0 && num_line_whitespace % 2 == 0 {
                        num_line_whitespace - 2
                    } else {
                        num_line_whitespace
                    };

                body_text_ident = ((list_item_whitespace as f64
                    / (indent_level * INDENT_SHIFT) as f64)
                    .ceil()) as i8;
            }

            new_indent_width = body_text_ident * INDENT_SHIFT + extra_spaces;
            last_action = Action::InsertBodyText;
        }

        let indent = usize::try_from(new_indent_width).unwrap();
        formatted += &format!("{}{}\n", " ".repeat(indent), trimmed_line);
    }

    formatted
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::{Path, PathBuf};

    fn format_file(path: PathBuf) -> String {
        let mut file = File::open(path).expect("Open file");
        let mut contents = String::new();
        // TODO BufReader
        file.read_to_string(&mut contents)
            .expect("Read file to string");

        format(&contents)
    }

    fn assert_equal(a: &str, b: &str) {
        if !(*a == *b) {
            let zipped_lines = a.lines().zip(b.lines());

            for lines in zipped_lines {
                if lines.0 == lines.1 {
                    println!("  {}", lines.0);
                } else {
                    println!("---");
                    println!("<!{}", lines.0);
                    println!(">!{}", lines.1);
                    println!("---");
                }
            }

            panic!("Strings are not equal");
        }
    }

    #[test]
    fn test_simple_case() {
        let formatted = format_file(Path::new("tests/1.input").to_path_buf());

        let mut expected = String::new();
        File::open("tests/1.expected")
            .expect("Open file")
            .read_to_string(&mut expected)
            .expect("Read to string");

        assert_equal(&formatted, &expected);
    }

    #[test]
    /// Formatting a text for a second time should not result in a different output
    fn test_format_twice() {
        let first_format = format_file(Path::new("tests/1.input").to_path_buf());
        let second_format = format(&first_format);

        assert_equal(&first_format, &second_format);
    }
}
