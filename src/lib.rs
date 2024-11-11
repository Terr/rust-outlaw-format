pub mod consts;

mod formatting;
mod parsing;

pub use formatting::{format_to_string, wrap_long_lines};
pub use parsing::parse_document;

pub fn format(contents: &str) -> String {
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

    fn last_block(&self) -> &Block {
        self.blocks
            .last()
            .expect("there should always be at least one Block")
    }

    fn last_block_mut(&mut self) -> &mut Block {
        self.blocks
            .last_mut()
            .expect("there should always be at least one Block")
    }

    fn find_latest_block_with_raw_indent(&self, num_indent: usize) -> Option<&Block> {
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
    is_before_first_header: bool,
}

impl Block {
    pub fn new(header: FormattedLine) -> Self {
        let is_before_first_header = header.is_empty();

        Block {
            header,
            contents: Vec::new(),
            is_before_first_header,
        }
    }

    pub fn add_line(&mut self, line: FormattedLine) {
        self.contents.push(line);
    }

    /// Returns the indentation level of the text contents the `Block` and sibling headers
    fn contents_indent_level(&self) -> usize {
        if self.is_before_first_header {
            0
        } else {
            self.header.indent_level + 1
        }
    }

    fn has_header(&self) -> bool {
        !self.header.is_empty()
    }

    fn raw_header_indent(&self) -> usize {
        self.header.original_raw.num_indent
    }

    fn last_line(&self) -> Option<&FormattedLine> {
        self.contents.iter().last()
    }

    fn find_previous_of(&self, line_type: LineType) -> Option<&FormattedLine> {
        self.contents
            .iter()
            .rev()
            .find(|line| line.line_type == line_type)
    }

    fn find_latest_line_with_raw_indent(&self, num_indent: usize) -> Option<&FormattedLine> {
        self.contents
            .iter()
            .rev()
            .find(|line| line.original_raw.num_indent == num_indent)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct RawLine {
    num_indent: usize,
    raw: String,
    trimmed: String,
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

    fn is_empty(&self) -> bool {
        self.raw.trim().is_empty()
    }

    fn is_list_item(&self) -> bool {
        matches!(
            LineType::from_raw(&self.trimmed),
            LineType::ListBulletPoint | LineType::ListTodoItem
        )
    }

    fn is_header(&self) -> bool {
        LineType::from_raw(&self.trimmed) == LineType::Header
    }

    fn contains_marker(&self) -> bool {
        self.trimmed
            .starts_with(consts::MARKER_FENCED_FILETYPE_BACKTICK)
            || self
                .trimmed
                .starts_with(consts::MARKER_FENCED_FILETYPE_TILDE)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct FormattedLine {
    contents: String,
    indent_level: usize,
    line_type: LineType,
    original_raw: RawLine,
}

impl FormattedLine {
    pub fn empty() -> Self {
        FormattedLine::default()
    }

    pub fn from_raw(raw_line: RawLine, indent_level: usize) -> Self {
        FormattedLine {
            contents: raw_line.trimmed.clone(),
            indent_level,
            line_type: LineType::from_raw(&raw_line.trimmed),
            original_raw: raw_line,
        }
    }

    fn is_empty(&self) -> bool {
        self.contents.len() == 0
    }

    fn is_list_item(&self) -> bool {
        matches!(
            self.line_type,
            LineType::ListBulletPoint | LineType::ListTodoItem | LineType::ListContinuousLine
        )
    }

    fn num_indent(&self) -> usize {
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
    /// Currently processing a wrapped line that is part of a list started on an earlier line
    ListContinuousLine,
    /// An item on a TODO list
    ListTodoItem,
    /// A line that starts with a '|' is considered to be preformatted, and *can* be longer than
    /// the maximum line length.
    Preformatted,
    /// A line that is prefixed with a '>'
    Quote,
}

impl LineType {
    /// Detects the type of the given `line` by looking at its first characters.
    /// Note that this function cannot determine if the line is a 'continuation line' in a bullet
    /// point list since that requires knowledge about the line preceding this one.
    fn from_raw(line: &str) -> Self {
        if line.starts_with(consts::PREFIX_HEADER) {
            LineType::Header
        } else if line.starts_with(consts::PREFIX_BULLET_POINT) {
            LineType::ListBulletPoint
        } else if line.starts_with(consts::PREFIX_TODO_ITEM) {
            LineType::ListTodoItem
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

    fn get_prefix(&self) -> &str {
        match self {
            Self::Header => consts::PREFIX_HEADER,
            Self::ListBulletPoint => consts::PREFIX_BULLET_POINT,
            Self::ListContinuousLine => consts::PREFIX_LIST_CONTINUATION,
            Self::ListTodoItem => consts::PREFIX_TODO_ITEM,
            Self::Preformatted => consts::PREFIX_PREFORMATTED,
            Self::Quote => consts::PREFIX_QUOTE,
            _ => "",
        }
    }

    fn get_prefix_length(&self) -> usize {
        match self {
            // '[ ] ' or '[x] '
            Self::ListTodoItem => 4,
            _ => self.get_prefix().len(),
        }
    }
}
