//use std::fs::File;
use std::io::{self, Read};
//use std::path::{Path, PathBuf};

static INDENT_SHIFT: u8 = 4;

fn main() {
    let mut stdin = io::stdin();
    let mut contents = String::new();
    stdin.read_to_string(&mut contents).expect("read stdin");

    //let input = File::open(path)?;
    //let buf_reader = io::BufReader::new(input);

    format(contents);
}

fn format(contents: String) {
    let mut indent_level = 0;
    let mut num_indent_whitespace = 0;
    let mut first_line_after_header = false;

    for line in contents.lines() {
        let trimmed_line = line.trim_start();
        let num_line_whitespace: u8 = (line.len() - trimmed_line.len()) as u8;
        let trimmed_line = trimmed_line.trim_end();

        // Write an empty line after a new header
        if first_line_after_header {
            println!();
            first_line_after_header = false;

            if trimmed_line == "" {
                continue;
            }
        }

        if trimmed_line.starts_with("===") {
            // Determine if this header is meant as the start of a deeper, equal or shallower
            // indentation

            if num_line_whitespace > num_indent_whitespace {
                indent_level += 1;
                num_indent_whitespace = num_line_whitespace;
            } else if num_line_whitespace < num_indent_whitespace {
                indent_level -= 1;
                num_indent_whitespace = num_line_whitespace;
            }

            println!(
                "{}{}",
                " ".repeat(((indent_level) * INDENT_SHIFT).into()),
                trimmed_line
            );

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

            println!(
                "{}{}",
                " ".repeat(((body_text_ident + indent_level) * INDENT_SHIFT).into()),
                trimmed_line
            );
        }
    }
}
