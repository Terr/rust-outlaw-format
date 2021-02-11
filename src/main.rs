use std::io::{self, Read};

static INDENT_SHIFT: u8 = 4;

fn main() {
    let mut stdin = io::stdin();
    let mut contents = String::new();
    stdin.read_to_string(&mut contents).expect("read stdin");

    //let input = File::open(path)?;
    //let buf_reader = io::BufReader::new(input);

    println!("{}", format(&contents));
}

fn format(contents: &str) -> String {
    let mut formatted = String::with_capacity(contents.len());

    let mut indent_level = 0;
    let mut num_indent_whitespace = 0;
    let mut first_line_after_header = false;

    for line in contents.lines() {
        let trimmed_line = line.trim_start();
        let num_line_whitespace: u8 = (line.len() - trimmed_line.len()) as u8;
        let trimmed_line = trimmed_line.trim_end();

        // Write an empty line after a new header
        if first_line_after_header {
            formatted += "\n";
            first_line_after_header = false;

            if trimmed_line == "" {
                continue;
            }
        }

        if trimmed_line.len() == 0 {
            formatted += "\n";
            continue;
        }

        let new_indent_width;
        if trimmed_line.starts_with("===") {
            // Determine if this header is meant as the start of a deeper, equal or shallower indentation

            if num_line_whitespace > num_indent_whitespace {
                indent_level += 1;
                num_indent_whitespace = num_line_whitespace;
            } else if num_line_whitespace < num_indent_whitespace {
                // This header can be on a parent, or even grandparent level
                // compared to the previous one, shifting
                // multiple identation levels to the left
                indent_level = num_line_whitespace / INDENT_SHIFT;
                num_indent_whitespace = num_line_whitespace;
            }

            new_indent_width = indent_level * INDENT_SHIFT;
            first_line_after_header = true;
        } else {
            let mut body_text_ident: u8 = 1;

            // Lists are allowed to have extra indentation
            if trimmed_line.starts_with('*') {
                if num_line_whitespace > num_indent_whitespace {
                    // Assumes that the type of indentation (spaces or tabs) used for this line was
                    //the same as what was used for the header
                    body_text_ident = num_line_whitespace
                        / (num_indent_whitespace + (body_text_ident * INDENT_SHIFT));
                }
            }

            new_indent_width = (body_text_ident + indent_level) * INDENT_SHIFT;
        }

        formatted += &format!("{}{}\n", " ".repeat(new_indent_width.into()), trimmed_line);
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

    #[test]
    fn test_simple_case() {
        let formatted = format_file(Path::new("tests/1.input").to_path_buf());

        let mut expected = String::new();
        File::open("tests/1.expected")
            .expect("Open file")
            .read_to_string(&mut expected)
            .expect("Read to string");

        assert_eq!(formatted, expected);
    }
}
