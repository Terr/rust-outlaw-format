use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use outlaw_format::format;

fn main() -> Result<(), String> {
    let contents = if env::args().len() > 1 {
        let path = env::args().nth(1).unwrap();
        let read_result = read_file(Path::new(&path));

        if let Err(err) = read_result {
            return match err.kind() {
                io::ErrorKind::NotFound => Err(format!("{}: File not found", path)),
                io::ErrorKind::PermissionDenied => Err(format!(
                    "{}: Could not read file because permission was denied",
                    path
                )),
                _ => Err(format!("{}: Read error: {}", path, err)),
            };
        }

        read_result.unwrap()
    } else {
        match read_stdin() {
            Ok(contents) => contents,
            Err(err) => return Err(format!("Could not read from stdin: {}", err)),
        }
    };

    print!("{}", format(&contents));

    Ok(())
}

fn read_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::with_capacity(fs::metadata(path)?.len() as usize);
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn read_stdin() -> io::Result<String> {
    let mut stdin = io::stdin();
    let mut contents = String::new();

    stdin.read_to_string(&mut contents)?;

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    use self::utils::*;

    #[test]
    fn format_empty_document() {
        let actual = format("");

        assert_equal(&actual, "");
    }

    #[test]
    fn format_simple_case() {
        let actual = format_file(Path::new("tests/simple.input"));
        let expected = read_file(Path::new("tests/simple.expected")).unwrap();

        assert_equal(&actual, &expected);
    }

    #[test]
    fn format_full_document() {
        let actual = format_file(Path::new("tests/1.input"));
        let expected = read_file(Path::new("tests/1.expected")).unwrap();

        assert_equal(&actual, &expected);
    }

    /// Formatting a text for a second time should not result in a different output
    #[test]
    fn format_full_document_twice() {
        let first_format = format_file(Path::new("tests/1.input"));
        let second_format = format(&first_format);

        assert_equal(&second_format, &first_format);
    }

    #[test]
    fn format_bullet_point_list() {
        let expected = read_file(Path::new("tests/bullet_points.expected")).unwrap();
        let actual = format_file(Path::new("tests/bullet_points.input"));

        assert_equal(&actual, &expected);
    }

    #[test]
    fn format_fenced_filetypes() {
        let expected = read_file(Path::new("tests/fenced_filetypes.expected")).unwrap();
        let actual = format_file(Path::new("tests/fenced_filetypes.input"));

        assert_equal(&actual, &expected);
    }

    #[test]
    fn wrapping_long_lines() {
        let expected = read_file(Path::new("tests/long_lines.expected")).unwrap();
        let first_format = format_file(Path::new("tests/long_lines.input"));
        let second_format = format(&first_format);

        assert_equal(&second_format, &expected);
    }

    mod utils {
        use std::path::Path;

        use super::*;

        pub(super) fn format_file(path: &Path) -> String {
            let contents = read_file(path).unwrap();

            format(&contents)
        }

        pub(super) fn assert_equal(actual: &str, expected: &str) {
            if *actual != *expected {
                let actual_lines = actual.lines().collect::<Vec<&str>>();
                let expected_lines = expected.lines().collect::<Vec<&str>>();

                eprintln!(
                    "Expected {} lines, got {}",
                    expected_lines.len(),
                    actual_lines.len(),
                );
                if actual_lines.len() != expected_lines.len() {
                    eprintln!("--- EXPECTED ---");
                    for line in expected_lines.iter() {
                        eprintln!("        '{}'", line);
                    }
                    eprintln!("--- ACTUAL ---");
                    for line in actual_lines.iter() {
                        eprintln!("        '{}'", line);
                    }
                } else {
                    let zipped_lines = actual.lines().zip(expected.lines());

                    for lines in zipped_lines {
                        if lines.0 == lines.1 {
                            eprintln!("        '{}'", lines.0);
                        } else {
                            eprintln!("---");
                            eprintln!("<ACTUAL '{}'", lines.0);
                            eprintln!("EXPECT> '{}'", lines.1);
                            eprintln!("---");
                        }
                    }
                }

                panic!("Strings are not equal");
            }
        }
    }
}
