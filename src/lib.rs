pub use formatting::{format_to_string, wrap_long_lines};
pub use parsing::parse_document;

pub mod consts;

mod formatting;
mod parsing;

pub fn format(contents: &str) -> String {
    let contents = contents;

    // Step 1: parse raw lines
    let mut document = parse_document(contents);

    // Step 2: wrapping of long lines
    for block in document.blocks.iter_mut() {
        wrap_long_lines(&mut block.contents, consts::MAX_LINE_LENGTH)
    }

    // Step 3: print formatted lines back into a string, adds extra newlines where needed
    format_to_string(&document)
}

#[derive(Debug)]
pub struct Document {
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn new() -> Self {
        let empty_header = FormattedLine::empty();
        let first_block = Block::new(empty_header);

        Document {
            blocks: vec![first_block],
        }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn last_block(&self) -> &Block {
        self.blocks
            .last()
            .expect("there should always be at least one Block")
    }

    pub fn last_block_mut(&mut self) -> &mut Block {
        self.blocks
            .last_mut()
            .expect("there should always be at least one Block")
    }

    pub fn find_latest_block_with_raw_indent(&self, num_indent: usize) -> Option<&Block> {
        self.blocks
            .iter()
            .rev()
            .find(|block| block.raw_header_indent() == num_indent)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Block {
    contents: Vec<FormattedLine>,
    header: FormattedLine,
}

impl Block {
    pub fn new(header: FormattedLine) -> Self {
        Block {
            header,
            contents: Vec::new(),
        }
    }

    pub fn add_line(&mut self, line: FormattedLine) {
        self.contents.push(line);
    }

    pub fn indent_level(&self) -> usize {
        self.header.indent_level
    }

    pub fn has_header(&self) -> bool {
        !self.header.is_empty()
    }

    pub fn raw_header_indent(&self) -> usize {
        self.header.original_raw.num_indent
    }

    pub fn last_line(&self) -> Option<&FormattedLine> {
        self.contents.iter().last()
    }

    pub fn find_previous_of(&self, line_type: LineType) -> Option<&FormattedLine> {
        self.contents
            .iter()
            .rev()
            .find(|line| line.line_type == line_type)
    }

    pub fn find_latest_line_with_raw_indent(&self, num_indent: usize) -> Option<&FormattedLine> {
        self.contents
            .iter()
            .rev()
            .find(|line| line.original_raw.num_indent == num_indent)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct RawLine {
    pub num_indent: usize,
    pub raw: String,
    pub trimmed: String,
}

impl RawLine {
    pub fn from_string(raw: &str) -> Self {
        let trimmed = raw.trim_start();
        let num_indent = raw.len() - trimmed.len();
        let trimmed = trimmed.trim_end();

        RawLine {
            num_indent,
            raw: raw.to_owned(),
            trimmed: trimmed.to_owned(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.raw.trim().is_empty()
    }

    pub fn is_bullet_point(&self) -> bool {
        LineType::from_raw(&self.trimmed) == LineType::ListBulletPoint
    }

    pub fn is_header(&self) -> bool {
        LineType::from_raw(&self.trimmed) == LineType::Header
    }

    pub fn contains_marker(&self) -> bool {
        self.trimmed
            .starts_with(consts::MARKER_FENCED_FILETYPE_BACKTICK)
            || self
                .trimmed
                .starts_with(consts::MARKER_FENCED_FILETYPE_TILDE)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct FormattedLine {
    indent_level: usize,
    pub line_type: LineType,
    pub contents: String,
    pub original_raw: RawLine,
}

impl FormattedLine {
    pub fn empty() -> Self {
        FormattedLine::default()
    }

    pub fn from_raw(raw_line: RawLine, indent_level: usize) -> Self {
        FormattedLine {
            indent_level,
            line_type: LineType::from_raw(&raw_line.trimmed),
            contents: raw_line.trimmed.clone(),
            original_raw: raw_line,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.contents.len() == 0
    }

    pub fn is_list_item(&self) -> bool {
        self.line_type == LineType::ListBulletPoint
            || self.line_type == LineType::ListContinuousLine
    }

    pub fn num_indent(&self) -> usize {
        self.indent_level * consts::INDENT_SHIFT
    }
}

#[derive(Debug, Eq, PartialEq, Default, Clone, Copy)]
pub enum LineType {
    #[default]
    Text,
    Header,
    /// Currently processing a list item that started with a '* '
    ListBulletPoint,
    /// Currently processing a wrapped line that is part of the previous ListBulletPoint
    ListContinuousLine,
    /// A line that starts with a '|' is considered to be preformatted, and can be longer than the
    /// maximum line length.
    Preformatted,
    /// A line that is prefixed with a '>'
    Quote,
}

impl LineType {
    /// Note that this function cannot determine if a line is a 'continuation line' in a bullet
    /// point list since that requires knowledge about the line preceding this one.
    pub fn from_raw(line: &str) -> Self {
        if line.starts_with(consts::PREFIX_HEADER) {
            LineType::Header
        } else if line.starts_with(consts::PREFIX_BULLET_POINT) {
            LineType::ListBulletPoint
        } else if line.starts_with(consts::MARKER_FENCED_FILETYPE_BACKTICK)
            || line.starts_with(consts::MARKER_FENCED_FILETYPE_TILDE)
            || line.starts_with(consts::PREFIX_PREFORMATTED)
        {
            LineType::Preformatted
        } else if line.starts_with(consts::PREFIX_QUOTE) {
            LineType::Quote
        } else {
            LineType::Text
        }
    }

    pub fn get_prefix(&self) -> &str {
        match self {
            Self::Header => consts::PREFIX_HEADER,
            Self::ListBulletPoint => consts::PREFIX_BULLET_POINT,
            Self::ListContinuousLine => consts::PREFIX_LIST_CONTINUATION,
            Self::Preformatted => consts::PREFIX_PREFORMATTED,
            Self::Quote => consts::PREFIX_QUOTE,
            _ => "",
        }
    }
}
