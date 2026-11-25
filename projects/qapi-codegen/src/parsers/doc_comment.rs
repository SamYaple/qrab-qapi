//! Documentation comments as QEMU's own parser reads them in each release
//! from 2.9, the first to give them a grammar: the rules they must follow
//! and the names they describe.
//!
//! Each release's parser reads a schema file as tokens. A comment that
//! starts with `##` opens a documentation comment, whose lines are comment
//! tokens up to a closing `##`. A fault is reported at the line QEMU names,
//! which is the comment line being read, or the comment's opening `##` for
//! the rules QEMU checks on the whole comment.
use super::common::{QapiString, is_whitespace};
use super::parser_version;
use crate::ParseErrorKind;

/// A rule of documentation comments, as QEMU's parser applies it in the
/// release being read.
///
/// ```
/// use qapi_codegen::{DocumentationError, ParseErrorKind};
///
/// let schema = "##\n# @Example:\n#\n# @member:\n##\n{ 'struct': 'Example', 'data': { 'member': 'int' } }\n";
/// let error = qapi_codegen::parse_schema_with_version(schema, (11, 1, 3)).unwrap_err();
/// assert_eq!(error.line, 4);
/// assert_eq!(
///     error.kind,
///     ParseErrorKind::Documentation(Box::new(DocumentationError::EmptySection {
///         section: "Member".into()
///     }))
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum DocumentationError {
    /// More than `##` on the line that opens a comment.
    #[error("junk after '##' at start of documentation comment")]
    JunkAtStart,
    /// More than `##` on the line that closes a comment.
    #[error("junk after '##' at end of documentation comment")]
    JunkAtEnd,
    /// Something other than a comment line before the closing `##`.
    #[error("documentation comment must end with '##'")]
    Unterminated,
    /// A comment line whose `#` is not followed by a space.
    #[error("missing space after #")]
    MissingSpace,
    /// A first line that starts with `@` but does not end with `:`.
    #[error("line should end with ':'")]
    SymbolWithoutColon,
    /// A first line `@:`, which names no definition.
    #[error("name required after '@'")]
    EmptySymbol,
    /// A member or feature description `@:`, which names nothing.
    #[error("invalid parameter name")]
    EmptyName,
    /// A member or feature described twice.
    #[error("'{name}' parameter name duplicated")]
    DuplicateName {
        /// The member or feature.
        name: String,
    },
    /// A member or feature description after a section that ends the
    /// descriptions.
    #[error("description of '@{name}:' follows a section")]
    DescriptionAfterSection {
        /// The member or feature.
        name: String,
    },
    /// A second `Since`, `Returns` or, from 9.0, `Errors` section.
    #[error("duplicated '{section}' section")]
    DuplicateSection {
        /// The section's tag.
        section: String,
    },
    /// A section or description without text.
    #[error("text required after '{section}:'")]
    EmptySection {
        /// The section's tag, or the member or feature, as the release
        /// names it.
        section: String,
    },
    /// A description `@name:` in a comment that documents no definition, or
    /// that is not at the start of a line of the comment's text.
    #[error("'{text}' not allowed in free-form documentation")]
    NotInFreeform {
        /// The description's text up to its colon.
        text: String,
    },
    /// An `#optional` tag in a member description (2.9 to 2.11).
    #[error("please drop the #optional tag")]
    OptionalTag,
    /// A second `Features:` line (from 9.0).
    #[error("duplicated 'Features:' line")]
    DuplicateFeatures,
    /// A `Features:` line without feature descriptions (from 9.0).
    #[error("feature descriptions expected")]
    NoFeatureDescriptions,
    /// A continuation line indented less than its section's text (from 5.2).
    #[error("unexpected de-indent (expected at least {expected} spaces)")]
    UnexpectedDeIndent {
        /// The indentation of the section's text.
        expected: usize,
    },
    /// A `=` heading in a definition's documentation (5.2 to 10.0).
    #[error("unexpected '=' markup in definition documentation")]
    HeadingInDefinition,
    /// A `=` heading after the first line of a comment (9.0 to 10.0).
    #[error("'=' heading must come first in a comment block")]
    HeadingNotFirst,
    /// A `Note` or `Example` section (from 9.1), which is rST markup now.
    #[error("the '{section}' section is no longer supported")]
    UnsupportedSection {
        /// The section's tag.
        section: String,
    },
    /// A line longer than 70 characters outside literal text (from 10.2).
    #[error("documentation line longer than 70 characters")]
    LongLine,
    /// One space after the end of a sentence outside literal text (from
    /// 10.2).
    #[error("use two spaces between sentences")]
    SingleSpace,
    /// A definition's documentation that no definition follows.
    #[error("documentation for '{symbol}' is not followed by the definition")]
    NotFollowedByDefinition {
        /// The definition the comment documents.
        symbol: String,
    },
    /// A comment that documents no definition, directly before one.
    #[error("definition documentation required")]
    DefinitionDocumentationRequired,
}

/// A fault that QEMU's parser reports while it reads documentation.
#[derive(Debug, Clone)]
pub(crate) struct Fault<'i> {
    /// The input from where QEMU reports the fault.
    pub(crate) at: &'i str,
    /// The input from the comment the fault belongs to: QEMU finds it no
    /// later than there.
    pub(crate) comment: &'i str,
    /// The rule that the comment breaks, or what is wrong with a token
    /// that interrupts it.
    pub(crate) kind: ParseErrorKind,
}

/// What a definition's documentation describes, as QEMU's parser reads it,
/// for the checks of the definition against it.
pub(crate) trait Describe<'i> {
    /// A member description, at its comment line.
    fn member(&mut self, _name: &str, _at: &'i str) {}
    /// A feature description (from 4.1), at its comment line.
    fn feature(&mut self, _name: &str, _at: &'i str) {}
    /// The `Returns` section, at its comment line from 9.0 and at the
    /// comment's opening `##` before, as QEMU reports it.
    fn returns(&mut self, _at: &'i str) {}
    /// The `Errors` section (from 9.0), at its comment line.
    fn errors(&mut self, _at: &'i str) {}
    /// From 10.0: how many sections follow the introduction, and whether
    /// each of them describes a member.
    fn sections(&mut self, _count: usize, _members: bool) {}
}

/// Reading only for the rules of the comment itself.
impl Describe<'_> for () {}

/// A member or feature that a comment describes.
struct Description<'i> {
    name: String,
    /// The comment line of the description.
    at: &'i str,
}

fn describes(descriptions: &[Description<'_>], name: &str) -> bool {
    descriptions
        .iter()
        .any(|description| description.name == name)
}

/// A documentation comment as QEMU's parser reads it.
#[derive(Default)]
struct DocComment<'i> {
    /// The definition it documents; `None` for free-form documentation.
    symbol: Option<String>,
    members: Vec<Description<'i>>,
    features: Vec<Description<'i>>,
    returns: Option<&'i str>,
    errors: Option<&'i str>,
    /// From 10.0: the sections after the introduction, and whether they
    /// all describe members.
    sections: Option<(usize, bool)>,
}

fn at_least(version: (u32, u32, u32)) -> bool {
    parser_version() >= version
}

/// QEMU's tokens, as its parser sees them while reading documentation.
#[derive(Clone, Copy)]
enum Token<'i> {
    /// A comment: the input from its `#` and its text up to the line end.
    Comment(&'i str, &'i str),
    /// Any other token, from its start.
    Other(&'i str),
    /// The end of the input, at the newline that ends it.
    End(&'i str),
}

impl<'i> Token<'i> {
    fn at(self) -> &'i str {
        match self {
            Self::Comment(at, _) | Self::Other(at) | Self::End(at) => at,
        }
    }
}

/// The token that starts at or after `input`, and the input after it. With
/// `skip_comments`, as QEMU's lexer does between expressions, comments that
/// do not start with `##` are skipped.
fn next_token(mut input: &str, skip_comments: bool) -> (Token<'_>, &str) {
    loop {
        let start = input.trim_start_matches(is_whitespace);
        if start.is_empty() {
            let end = input.strip_suffix('\n').unwrap_or(input);
            let end = end.strip_suffix('\r').unwrap_or(end);
            return (Token::End(&input[end.len()..]), start);
        }
        if start.starts_with('#') {
            let length = start.find(['\r', '\n']).unwrap_or(start.len());
            let (text, rest) = start.split_at(length);
            if skip_comments && !text.starts_with("##") {
                input = rest;
                continue;
            }
            return (Token::Comment(start, text), rest);
        }
        return (Token::Other(start), start);
    }
}

fn fault<'i>(at: &'i str, comment: &'i str, error: DocumentationError) -> Fault<'i> {
    Fault {
        at,
        comment,
        kind: ParseErrorKind::Documentation(Box::new(error)),
    }
}

/// The comment that starts at the `##` of `input`, as the current release's
/// parser reads it: the definition it documents (`None` for free-form
/// documentation) and the input after its closing `##`. What it describes
/// goes to `describe`.
pub(crate) fn read<'i>(
    input: &'i str,
    describe: &mut dyn Describe<'i>,
) -> Result<(Option<String>, &'i str), Fault<'i>> {
    let (token, rest) = next_token(input, false);
    let mut reader = Reader {
        comment: token.at(),
        token,
        rest,
        literal: None,
    };
    let Token::Comment(at, text) = token else {
        return Err(fault(
            token.at(),
            token.at(),
            DocumentationError::Unterminated,
        ));
    };
    if text != "##" {
        return Err(fault(at, at, DocumentationError::JunkAtStart));
    }
    let doc = if at_least((9, 0, 0)) {
        reader.current()?
    } else {
        reader.legacy()?
    };
    for description in &doc.members {
        describe.member(&description.name, description.at);
    }
    for description in &doc.features {
        describe.feature(&description.name, description.at);
    }
    if let Some(at) = doc.returns {
        describe.returns(at);
    }
    if let Some(at) = doc.errors {
        describe.errors(at);
    }
    if let Some((count, members)) = doc.sections {
        describe.sections(count, members);
    }
    Ok((doc.symbol, reader.rest))
}

struct Reader<'i> {
    /// The input from the comment's opening `##`.
    comment: &'i str,
    /// The current token and the input after it.
    token: Token<'i>,
    rest: &'i str,
    /// From 10.2: the indentation of literal text being read, `Some(0)`
    /// before its first line.
    literal: Option<usize>,
}

/// Characters, as Python counts them.
fn chars(text: &str) -> usize {
    text.chars().count()
}

fn indentation(line: &str) -> usize {
    chars(line) - chars(line.trim_start_matches(is_whitespace))
}

/// Python's `line[n:]`, in characters.
fn skip_chars(line: &str, n: usize) -> &str {
    let index = line
        .char_indices()
        .nth(n)
        .map_or(line.len(), |(index, _)| index);
    &line[index..]
}

/// `re.match(r'@([^:]*): *', line)`: the name and the text after the match.
fn at_name_colon(line: &str) -> Option<(&str, &str)> {
    let (name, rest) = line.strip_prefix('@')?.split_once(':')?;
    Some((name, rest.trim_start_matches(' ')))
}

/// `re.match(r'(@\S+:)', line)`: the description up to its colon.
fn freeform_description(line: &str) -> Option<&str> {
    if !line.starts_with('@') {
        return None;
    }
    let run = &line[..line.find(is_whitespace).unwrap_or(line.len())];
    let colon = run.rfind(':')?;
    (colon >= 2).then(|| &run[..=colon])
}

/// The section tags of 2.9 to 8.0, as the first word of a line.
const TAGS: [&str; 7] = [
    "Returns:",
    "Since:",
    "Note:",
    "Notes:",
    "Example:",
    "Examples:",
    "TODO:",
];

/// `re.match(r'(Returns|Since|Notes?|Examples?|TODO): *', line)` (8.1 and
/// 8.2) and, from 9.0, with `Errors` and, from 9.1, not before `::`.
fn section_tag(line: &str) -> Option<(&str, &str)> {
    let errors = at_least((9, 0, 0));
    [
        "Returns", "Errors", "Since", "Notes", "Note", "Examples", "Example", "TODO",
    ]
    .into_iter()
    .filter(|tag| errors || *tag != "Errors")
    .find_map(|tag| {
        let rest = line.strip_prefix(tag)?.strip_prefix(':')?;
        if at_least((9, 1, 0)) && rest.starts_with(':') {
            return None;
        }
        Some((tag, rest.trim_start_matches(' ')))
    })
}

/// Whether a 10.2 line breaks the length or sentence-spacing rule: the
/// error and, for spacing, the index in characters of the punctuation.
fn line_format(line: &str) -> Option<(DocumentationError, Option<usize>)> {
    if chars(line) + 2 > 70 && !is_url_line(line) {
        return Some((DocumentationError::LongLine, None));
    }
    let text: Vec<char> = line.chars().collect();
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let capital = |index: usize| {
        text.get(index) == Some(&' ')
            && text
                .get(index + 1)
                .is_some_and(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '(')
    };
    let mut index = 0;
    while index < text.len() {
        // r'(\be\.g\.|^ *\d\.|([.!?])) [A-Z0-9(]', in its order.
        let boundary = index == 0 || !word(text[index - 1]);
        if boundary && text[index..].starts_with(&['e', '.', 'g', '.']) && capital(index + 4) {
            index += 6;
            continue;
        }
        if index == 0 {
            let spaces = text.iter().take_while(|c| **c == ' ').count();
            if text.get(spaces).is_some_and(|c| c.is_numeric())
                && text.get(spaces + 1) == Some(&'.')
                && capital(spaces + 2)
            {
                index = spaces + 4;
                continue;
            }
        }
        if matches!(text[index], '.' | '!' | '?') && capital(index + 1) {
            return Some((DocumentationError::SingleSpace, Some(index)));
        }
        index += 1;
    }
    None
}

/// `re.match(r' *(https?|ftp)://[^ ]*$', line)`.
fn is_url_line(line: &str) -> bool {
    let line = line.trim_start_matches(' ');
    ["http://", "https://", "ftp://"].iter().any(|scheme| {
        line.strip_prefix(scheme)
            .is_some_and(|rest| !rest.contains(' '))
    })
}

impl<'i> Reader<'i> {
    fn fail<T>(&self, error: DocumentationError) -> Result<T, Fault<'i>> {
        Err(fault(self.token.at(), self.comment, error))
    }

    /// QEMU's `accept(False)`: the next token, comments included.
    fn accept(&mut self) {
        (self.token, self.rest) = next_token(self.rest, false);
    }

    /// The current token as a comment line, or the fault of a comment that
    /// ends without `##`.
    fn comment_text(&self) -> Result<(&'i str, &'i str), Fault<'i>> {
        match self.token {
            Token::Comment(at, text) => Ok((at, text)),
            Token::Other(at) => match super::common::token_error(at) {
                Some((at, kind)) => Err(Fault {
                    at,
                    comment: self.comment,
                    kind,
                }),
                None => self.fail(DocumentationError::Unterminated),
            },
            Token::End(_) => self.fail(DocumentationError::Unterminated),
        }
    }

    /// The end of the comment, if the current token closes it.
    fn closing(&self, text: &str) -> Result<bool, Fault<'i>> {
        if !text.starts_with("##") {
            return Ok(false);
        }
        if text != "##" {
            return self.fail(DocumentationError::JunkAtEnd);
        }
        Ok(true)
    }

    // ─── 2.9 to 8.2 ───────────────────────────────────────────────

    fn legacy(&mut self) -> Result<DocComment<'i>, Fault<'i>> {
        let mut doc = Legacy::default();
        self.accept();
        loop {
            let (at, text) = self.comment_text()?;
            if self.closing(text)? {
                doc.end_comment(self)?;
                return Ok(doc.into_comment(self.comment));
            }
            if at_least((5, 2, 0)) && text.starts_with("# =") {
                if doc.symbol.is_some() {
                    return self.fail(DocumentationError::HeadingInDefinition);
                }
                if !doc.body_text().is_empty() {
                    doc.end_comment(self)?;
                    doc = Legacy::default();
                }
            }
            doc.append(self, at, text)?;
            self.accept();
        }
    }

    // ─── 9.0 and later ────────────────────────────────────────────

    /// QEMU's `get_doc_line`: `None` at the closing `##`.
    fn line(&mut self) -> Result<Option<(&'i str, String)>, Fault<'i>> {
        let (at, text) = self.comment_text()?;
        if self.closing(text)? {
            self.literal = None;
            return Ok(None);
        }
        if text == "#" {
            return Ok(Some((at, String::new())));
        }
        if !text[1..].starts_with(' ') {
            return self.fail(DocumentationError::MissingSpace);
        }
        let line = text[2..].trim_end_matches(is_whitespace).to_owned();
        if at_least((10, 2, 0)) {
            let opens_literal = {
                let rest = line
                    .strip_prefix("..")
                    .and_then(|rest| {
                        let trimmed = rest.trim_start_matches(' ');
                        (trimmed.len() < rest.len())
                            .then(|| trimmed.strip_prefix("qmp-example"))
                            .flatten()
                    })
                    .unwrap_or(&line);
                rest.trim_start_matches(' ') == "::"
            };
            if opens_literal {
                self.literal = Some(0);
            } else if let Some(indent) = self.literal
                && !line.is_empty()
            {
                let current = indentation(&line);
                if indent == 0 {
                    self.literal = Some(current);
                } else if current < indent {
                    self.literal = None;
                }
            }
            if self.literal.is_none()
                && let Some((error, punctuation)) = line_format(&line)
            {
                let at = match punctuation {
                    // QEMU points after the punctuation, counting from the
                    // start of the line as though `# ` began it.
                    Some(index) => {
                        let source = self.line_start(at);
                        skip_chars(source, index + 3)
                    }
                    None => at,
                };
                return Err(fault(at, self.comment, error));
            }
        }
        Ok(Some((at, line)))
    }

    /// The input from the start of the line of `at`, within the comment.
    fn line_start(&self, at: &'i str) -> &'i str {
        let offset = self.comment.len() - at.len();
        let before = &self.comment[..offset];
        let start = before.rfind(['\r', '\n']).map_or(0, |index| index + 1);
        &self.comment[start..]
    }

    fn next_line(&mut self) -> Result<Option<(&'i str, String)>, Fault<'i>> {
        self.accept();
        self.line()
    }

    /// QEMU's `get_doc_indented`: the text of a section after its first
    /// line, and the line after it.
    fn indented(&mut self, doc: &mut Current<'i>) -> Result<Option<(&'i str, String)>, Fault<'i>> {
        let mut line = self.next_line()?;
        while let Some((_, text)) = &line
            && text.is_empty()
        {
            line = self.next_line()?;
        }
        let Some((_, text)) = &line else {
            return Ok(line);
        };
        let indent = indentation(text);
        if indent == 0 {
            return Ok(line);
        }
        doc.text();
        let mut previous_blank = false;
        loop {
            let line = self.next_line()?;
            let Some((_, text)) = &line else {
                return Ok(line);
            };
            if at_name_colon(text).is_some() {
                return Ok(line);
            }
            if !text.is_empty() && indentation(text) < indent {
                if previous_blank {
                    return Ok(line);
                }
                return self.fail(DocumentationError::UnexpectedDeIndent { expected: indent });
            }
            // QEMU marks every line after the first as following a blank.
            previous_blank = true;
        }
    }

    /// QEMU's `get_doc_paragraph`.
    fn paragraph(&mut self) -> Result<Option<(&'i str, String)>, Fault<'i>> {
        loop {
            let line = self.next_line()?;
            match &line {
                Some((_, text)) if !text.is_empty() => {}
                _ => return Ok(line),
            }
        }
    }

    fn current(&mut self) -> Result<DocComment<'i>, Fault<'i>> {
        let before_10_1 = !at_least((10, 1, 0));
        let mut doc = Current {
            start: self.comment,
            comment: DocComment::default(),
            sections: Vec::new(),
            since: false,
        };
        let mut line = self.next_line()?;
        match &line {
            Some((_, text)) if text.starts_with('@') => {
                if !text.ends_with(':') {
                    return self.fail(DocumentationError::SymbolWithoutColon);
                }
                let symbol = &text[1..text.len() - 1];
                if symbol.is_empty() {
                    return self.fail(DocumentationError::EmptySymbol);
                }
                doc.comment.symbol = Some(symbol.to_owned());
                line = if at_least((11, 1, 0)) {
                    doc.sections.push(Section::untagged(self.token.at()));
                    self.indented(&mut doc)?
                } else {
                    self.next_line()?
                };
                let mut no_more_args = false;
                while line.is_some() {
                    while let Some((_, text)) = &line
                        && text.is_empty()
                    {
                        line = self.next_line()?;
                    }
                    let Some((at, text)) = line.clone() else {
                        break;
                    };
                    if text == "Features:" {
                        if !doc.comment.features.is_empty() {
                            return self.fail(DocumentationError::DuplicateFeatures);
                        }
                        line = self.next_line()?;
                        while let Some((_, text)) = &line
                            && text.is_empty()
                        {
                            line = self.next_line()?;
                        }
                        while let Some((at, text)) = line.clone()
                            && let Some((name, rest)) = at_name_colon(&text)
                        {
                            doc.describe(at, name, true)?;
                            if !rest.is_empty() {
                                doc.text();
                            }
                            line = self.indented(&mut doc)?;
                        }
                        if doc.comment.features.is_empty() {
                            return self.fail(DocumentationError::NoFeatureDescriptions);
                        }
                        no_more_args = true;
                    } else if let Some((name, _)) = at_name_colon(&text) {
                        if no_more_args {
                            return self.fail(DocumentationError::DescriptionAfterSection {
                                name: name.to_owned(),
                            });
                        }
                        while let Some((at, text)) = line.clone()
                            && let Some((name, rest)) = at_name_colon(&text)
                        {
                            doc.describe(at, name, false)?;
                            if !rest.is_empty() {
                                doc.text();
                            }
                            line = self.indented(&mut doc)?;
                        }
                        no_more_args = true;
                    } else if let Some((tag, rest)) = section_tag(&text) {
                        if at_least((9, 1, 0))
                            && (tag.starts_with("Note") || tag.starts_with("Example"))
                        {
                            return self.fail(DocumentationError::UnsupportedSection {
                                section: tag.to_owned(),
                            });
                        }
                        doc.tagged(at, tag)?;
                        if !rest.is_empty() {
                            doc.text();
                        }
                        line = self.indented(&mut doc)?;
                        no_more_args = true;
                    } else if before_10_1 && text.starts_with('=') {
                        return self.fail(DocumentationError::HeadingInDefinition);
                    } else {
                        doc.untagged(at);
                        doc.text();
                        line = self.paragraph()?;
                    }
                }
            }
            _ => {
                let mut first = true;
                while let Some((_, text)) = &line {
                    if let Some((name, _)) = at_name_colon(text) {
                        return self.fail(DocumentationError::NotInFreeform {
                            text: format!("@{name}:"),
                        });
                    }
                    if before_10_1 && text.starts_with('=') && !first {
                        return self.fail(DocumentationError::HeadingNotFirst);
                    }
                    line = self.next_line()?;
                    first = false;
                }
            }
        }
        doc.end()
    }
}

/// A section of a 9.0 and later comment, for the check of its text.
struct Section<'i> {
    /// Its tag as the release names it, `None` when untagged.
    tag: Option<String>,
    at: &'i str,
    text: bool,
}

impl<'i> Section<'i> {
    fn untagged(at: &'i str) -> Self {
        Self {
            tag: None,
            at,
            text: false,
        }
    }
}

struct Current<'i> {
    /// The input from the comment's opening `##`.
    start: &'i str,
    comment: DocComment<'i>,
    sections: Vec<Section<'i>>,
    since: bool,
}

impl<'i> Current<'i> {
    /// Text for the last section.
    fn text(&mut self) {
        if let Some(section) = self.sections.last_mut() {
            section.text = true;
        }
    }

    /// A fault that QEMU reports at the line of a section.
    fn fail<T>(&self, at: &'i str, error: DocumentationError) -> Result<T, Fault<'i>> {
        Err(fault(at, self.start, error))
    }

    fn describe(&mut self, at: &'i str, name: &str, feature: bool) -> Result<(), Fault<'i>> {
        if name.is_empty() {
            return self.fail(at, DocumentationError::EmptyName);
        }
        let list = if feature {
            &mut self.comment.features
        } else {
            &mut self.comment.members
        };
        if describes(list, name) {
            let name = name.to_owned();
            return Err(fault(
                at,
                self.start,
                DocumentationError::DuplicateName { name },
            ));
        }
        list.push(Description {
            name: name.to_owned(),
            at,
        });
        let tag = if at_least((10, 0, 0)) {
            if feature { "Feature" } else { "Member" }.to_owned()
        } else {
            format!("@{name}")
        };
        self.sections.push(Section {
            tag: Some(tag),
            at,
            text: false,
        });
        Ok(())
    }

    fn tagged(&mut self, at: &'i str, tag: &str) -> Result<(), Fault<'i>> {
        let tag = if at_least((10, 0, 0)) && tag == "TODO" {
            "Todo"
        } else {
            tag
        };
        let slot = match tag {
            "Returns" => Some(self.comment.returns.is_some()),
            "Errors" => Some(self.comment.errors.is_some()),
            "Since" => Some(self.since),
            _ => None,
        };
        if slot == Some(true) {
            let section = tag.to_owned();
            return self.fail(at, DocumentationError::DuplicateSection { section });
        }
        match tag {
            "Returns" => self.comment.returns = Some(at),
            "Errors" => self.comment.errors = Some(at),
            "Since" => self.since = true,
            _ => {}
        }
        self.sections.push(Section {
            tag: Some(tag.to_owned()),
            at,
            text: false,
        });
        Ok(())
    }

    /// QEMU's `ensure_untagged_section`: text after a tagged section
    /// starts an untagged one.
    fn untagged(&mut self, at: &'i str) {
        if self
            .sections
            .last()
            .is_none_or(|section| section.tag.is_some())
        {
            self.sections.push(Section::untagged(at));
        }
    }

    /// QEMU's `QAPIDoc.end`: every tagged section needs text.
    fn end(mut self) -> Result<DocComment<'i>, Fault<'i>> {
        let empty = self
            .sections
            .iter()
            .find(|section| section.tag.is_some() && !section.text);
        if let Some(Section {
            tag: Some(tag), at, ..
        }) = empty
        {
            return self.fail(
                at,
                DocumentationError::EmptySection {
                    section: tag.clone(),
                },
            );
        }
        // Untagged text first is the introduction.
        let after = match self.sections.first() {
            Some(Section { tag: None, .. }) => &self.sections[1..],
            _ => &self.sections[..],
        };
        let members = after
            .iter()
            .all(|section| section.tag.as_deref() == Some("Member"));
        self.comment.sections = at_least((10, 0, 0)).then_some((after.len(), members));
        Ok(self.comment)
    }
}

/// The part of a 2.9 to 8.2 comment being read.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Part {
    #[default]
    Body,
    Members,
    Features,
    Sections,
}

/// The current section of a 2.9 to 8.2 comment.
#[derive(Default)]
struct LegacySection {
    /// `Body` for the body, `Members`/`Features` for descriptions, and
    /// `Sections` for the others.
    part: Part,
    /// Its name: the member or feature, or the tag; `Some("")` or `None`
    /// for an untagged section.
    name: Option<String>,
    /// The lines added (2.9 to 2.11, which compared them) and the text.
    lines: Vec<String>,
    text: String,
    /// From 5.2, the indentation stripped from its lines (`None`: not yet
    /// known, from 8.1).
    indent: Option<usize>,
    /// Whether the section has ended.
    ended: bool,
}

impl LegacySection {
    fn named(part: Part, name: Option<String>, indent: Option<usize>) -> Self {
        Self {
            part,
            name,
            indent,
            ..Self::default()
        }
    }

    fn is_named(&self) -> bool {
        self.name.as_ref().is_some_and(|name| !name.is_empty())
    }

    /// QEMU's `Section.append`.
    fn append<'i>(&mut self, reader: &Reader<'i>, line: &str) -> Result<(), Fault<'i>> {
        let mut line = line.to_owned();
        if at_least((8, 1, 0)) {
            line.truncate(line.trim_end_matches(is_whitespace).len());
        }
        if at_least((5, 2, 0)) && !line.is_empty() {
            let indent = indentation(&line);
            match self.indent {
                None if at_least((8, 1, 0)) => {
                    if !self.text.is_empty() {
                        self.indent = Some(indent);
                    }
                }
                Some(expected) if indent < expected => {
                    return reader.fail(DocumentationError::UnexpectedDeIndent { expected });
                }
                _ => {}
            }
            line = skip_chars(&line, self.indent.unwrap_or(0)).to_owned();
        }
        self.text.push_str(line.trim_end_matches(is_whitespace));
        self.text.push('\n');
        self.lines.push(line);
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.text.trim_matches(is_whitespace).is_empty()
    }

    /// A blank line ends the current text: QEMU's test whether the section
    /// ends in a blank line.
    fn after_blank(&self) -> bool {
        if at_least((2, 12, 0)) {
            self.text.ends_with("\n\n")
        } else {
            self.lines.last().is_some_and(String::is_empty)
        }
    }
}

struct Legacy<'i> {
    symbol: Option<String>,
    /// The body's text, once another section is current.
    body: Option<String>,
    members: Vec<Description<'i>>,
    features: Vec<Description<'i>>,
    /// The names of the sections other than the body and descriptions.
    sections: Vec<Option<String>>,
    returns: bool,
    section: LegacySection,
    /// The part whose lines are being read (from 4.1).
    part: Part,
}

impl Default for Legacy<'_> {
    fn default() -> Self {
        Self {
            symbol: None,
            body: None,
            members: Vec::new(),
            features: Vec::new(),
            sections: Vec::new(),
            returns: false,
            // The body strips no indentation.
            section: LegacySection::named(Part::Body, None, Some(0)),
            part: Part::Body,
        }
    }
}

impl<'i> Legacy<'i> {
    fn body_text(&self) -> &str {
        match &self.body {
            Some(text) => text,
            None => &self.section.text,
        }
    }

    fn into_comment(self, comment: &'i str) -> DocComment<'i> {
        DocComment {
            symbol: self.symbol,
            members: self.members,
            features: self.features,
            returns: self.returns.then_some(comment),
            errors: None,
            sections: None,
        }
    }

    /// QEMU's `QAPIDoc.append`.
    fn append(&mut self, reader: &Reader<'i>, at: &'i str, text: &'i str) -> Result<(), Fault<'i>> {
        let line = &text[1..];
        if line.is_empty() {
            return self.freeform(reader, "");
        }
        let Some(line) = line.strip_prefix(' ') else {
            return reader.fail(DocumentationError::MissingSpace);
        };
        if !at_least((4, 1, 0)) {
            if self.symbol.is_some() {
                return self.symbol_line(reader, at, line);
            }
            if self.body_text().is_empty() && line.starts_with('@') {
                return self.start_symbol(reader, line);
            }
            return self.freeform(reader, line);
        }
        match self.part {
            Part::Body => self.body_line(reader, at, line),
            Part::Members | Part::Features => self.description_line(reader, at, line),
            Part::Sections => self.section_line(reader, line),
        }
    }

    fn start_symbol(&mut self, reader: &Reader<'i>, line: &str) -> Result<(), Fault<'i>> {
        let Some(symbol) = line
            .strip_prefix('@')
            .and_then(|line| line.strip_suffix(':'))
        else {
            return reader.fail(DocumentationError::SymbolWithoutColon);
        };
        if symbol.is_empty() {
            return reader.fail(DocumentationError::EmptySymbol);
        }
        self.symbol = Some(symbol.to_owned());
        Ok(())
    }

    /// The first word of a line, as 2.9 to 8.0 split it.
    fn first_word(line: &str) -> &str {
        line.split(' ').next().unwrap_or(line)
    }

    fn is_description(line: &str) -> bool {
        if at_least((8, 1, 0)) {
            at_name_colon(line).is_some()
        } else {
            let word = Self::first_word(line);
            word.starts_with('@') && word.ends_with(':') && word.len() >= 2
        }
    }

    fn is_tag(line: &str) -> bool {
        if at_least((8, 1, 0)) {
            section_tag(line).is_some()
        } else {
            TAGS.contains(&Self::first_word(line))
        }
    }

    /// 2.9 to 4.0: QEMU's `_append_symbol_line`.
    fn symbol_line(
        &mut self,
        reader: &Reader<'i>,
        at: &'i str,
        line: &'i str,
    ) -> Result<(), Fault<'i>> {
        let word = Self::first_word(line);
        let rest = line.get(word.len() + 1..).unwrap_or("");
        if word.starts_with('@') && word.ends_with(':') && word.len() >= 2 {
            self.start_description(reader, at, &word[1..word.len() - 1], Part::Members, None)?;
            return self.freeform(reader, rest);
        }
        if TAGS.contains(&word) {
            self.start_section(reader, Some(word[..word.len() - 1].to_owned()), None)?;
            return self.freeform(reader, rest);
        }
        self.freeform(reader, line)
    }

    /// From 4.1: QEMU's `_append_body_line`.
    fn body_line(
        &mut self,
        reader: &Reader<'i>,
        at: &'i str,
        line: &'i str,
    ) -> Result<(), Fault<'i>> {
        if self.symbol.is_none() && self.body_text().is_empty() && line.starts_with('@') {
            return self.start_symbol(reader, line);
        }
        if self.symbol.is_some() {
            if Self::is_description(line) {
                self.part = Part::Members;
                return self.description_line(reader, at, line);
            }
            if line == "Features:" {
                self.part = Part::Features;
                return Ok(());
            }
            if Self::is_tag(line) {
                self.part = Part::Sections;
                return self.section_line(reader, line);
            }
        }
        self.freeform(reader, &self.strip_before_5_2(line))
    }

    fn strip_before_5_2(&self, line: &str) -> String {
        if at_least((5, 2, 0)) {
            line.to_owned()
        } else {
            line.trim_matches(is_whitespace).to_owned()
        }
    }

    /// The description's name and text, and from 5.2 to 8.0 the
    /// indentation of its text.
    fn split_description(line: &str) -> (&str, String, Option<usize>) {
        if at_least((8, 1, 0)) {
            let (name, rest) = at_name_colon(line).unwrap_or_default();
            return (name, rest.to_owned(), None);
        }
        let word = Self::first_word(line);
        let name = &word[1..word.len() - 1];
        if at_least((5, 2, 0)) {
            // re.match(r'@\S*:\s*', line).end()
            let run = &line[..line.find(is_whitespace).unwrap_or(line.len())];
            let colon = run.rfind(':').unwrap_or(0);
            let after = &line[colon + 1..];
            let end = chars(&line[..colon + 1]) + indentation(after);
            let rest = skip_chars(line, end);
            if rest.is_empty() {
                return (name, String::new(), Some(0));
            }
            return (name, format!("{}{rest}", " ".repeat(end)), Some(end));
        }
        (
            name,
            line.get(word.len() + 1..).unwrap_or("").to_owned(),
            None,
        )
    }

    /// From 4.1: QEMU's `_append_args_line` and `_append_features_line`.
    fn description_line(
        &mut self,
        reader: &Reader<'i>,
        at: &'i str,
        line: &'i str,
    ) -> Result<(), Fault<'i>> {
        if Self::is_description(line) {
            let (name, rest, indent) = Self::split_description(line);
            self.start_description(reader, at, name, self.part, indent)?;
            return self.freeform(reader, &self.strip_before_5_2(&rest));
        }
        if Self::is_tag(line) {
            self.part = Part::Sections;
            return self.section_line(reader, line);
        }
        if self.section.after_blank() && line.chars().next().is_some_and(|c| !is_whitespace(c)) {
            if self.part == Part::Members && line == "Features:" {
                self.part = Part::Features;
            } else {
                self.start_section(reader, None, Some(0))?;
                self.part = Part::Sections;
                self.section_line(reader, line)?;
            }
            return Ok(());
        }
        self.freeform(reader, &self.strip_before_5_2(line))
    }

    /// From 4.1: QEMU's `_append_various_line`.
    fn section_line(&mut self, reader: &Reader<'i>, line: &str) -> Result<(), Fault<'i>> {
        if Self::is_description(line) {
            let name = if at_least((8, 1, 0)) {
                at_name_colon(line).unwrap_or_default().0
            } else {
                let word = Self::first_word(line);
                &word[1..word.len() - 1]
            };
            return reader.fail(DocumentationError::DescriptionAfterSection {
                name: name.to_owned(),
            });
        }
        let mut line = line.to_owned();
        if Self::is_tag(&line) {
            let (tag, rest, indent) = if at_least((8, 1, 0)) {
                let (tag, rest) = section_tag(&line).unwrap_or_default();
                (tag.to_owned(), rest.to_owned(), None)
            } else {
                let word = Self::first_word(&line);
                let tag = word[..word.len() - 1].to_owned();
                if at_least((5, 2, 0)) {
                    // re.match(r'\S*:\s*', line).end()
                    let run = &line[..line.find(is_whitespace).unwrap_or(line.len())];
                    let colon = run.rfind(':').unwrap_or(0);
                    let end = chars(&line[..colon + 1]) + indentation(&line[colon + 1..]);
                    let rest = skip_chars(&line, end);
                    if rest.is_empty() {
                        (tag, String::new(), Some(0))
                    } else {
                        (tag, format!("{}{rest}", " ".repeat(end)), Some(end))
                    }
                } else {
                    let rest = line.get(word.len() + 1..).unwrap_or("").to_owned();
                    (tag, rest, Some(0))
                }
            };
            self.start_section(reader, Some(tag), indent)?;
            line = rest;
        }
        if !at_least((5, 2, 0))
            && !self
                .section
                .name
                .as_ref()
                .is_some_and(|name| name.starts_with("Example"))
        {
            line = line.trim_matches(is_whitespace).to_owned();
        }
        self.freeform(reader, &line)
    }

    fn start_description(
        &mut self,
        reader: &Reader<'i>,
        at: &'i str,
        name: &str,
        part: Part,
        indent: Option<usize>,
    ) -> Result<(), Fault<'i>> {
        if name.is_empty() {
            return reader.fail(DocumentationError::EmptyName);
        }
        let list = if part == Part::Features {
            &self.features
        } else {
            &self.members
        };
        if describes(list, name) {
            return reader.fail(DocumentationError::DuplicateName {
                name: name.to_owned(),
            });
        }
        if !self.sections.is_empty() {
            // 2.9 to 4.0; later releases switch to reading sections first.
            return reader.fail(DocumentationError::DescriptionAfterSection {
                name: name.to_owned(),
            });
        }
        let indent = if at_least((8, 1, 0)) {
            None
        } else {
            Some(indent.unwrap_or(0))
        };
        self.switch(
            reader,
            LegacySection::named(part, Some(name.to_owned()), indent),
        )?;
        let description = Description {
            name: name.to_owned(),
            at,
        };
        if part == Part::Features {
            self.features.push(description);
        } else {
            self.members.push(description);
        }
        Ok(())
    }

    fn start_section(
        &mut self,
        reader: &Reader<'i>,
        name: Option<String>,
        indent: Option<usize>,
    ) -> Result<(), Fault<'i>> {
        if let Some(name) = &name
            && matches!(name.as_str(), "Returns" | "Since")
            && self
                .sections
                .iter()
                .any(|section| section.as_ref() == Some(name))
        {
            return reader.fail(DocumentationError::DuplicateSection {
                section: name.clone(),
            });
        }
        // An untagged section: '' before 2.12, None after.
        let name = name.or_else(|| (!at_least((2, 12, 0))).then(String::new));
        let indent = match (&name, at_least((8, 1, 0))) {
            (Some(name), true) if !name.is_empty() => None,
            _ => Some(indent.unwrap_or(0)),
        };
        if name.as_deref() == Some("Returns") {
            self.returns = true;
        }
        self.switch(
            reader,
            LegacySection::named(Part::Sections, name.clone(), indent),
        )?;
        self.sections.push(name);
        Ok(())
    }

    /// QEMU's `_end_section` and, from 6.2, `_switch_section`.
    fn switch(&mut self, reader: &Reader<'i>, next: LegacySection) -> Result<(), Fault<'i>> {
        self.end_section(reader)?;
        let previous = std::mem::replace(&mut self.section, next);
        if previous.part == Part::Body && self.body.is_none() {
            self.body = Some(previous.text);
        }
        Ok(())
    }

    fn end_section(&mut self, reader: &Reader<'i>) -> Result<(), Fault<'i>> {
        if self.section.ended {
            return Ok(());
        }
        if self.section.part != Part::Body && self.section.is_named() && self.section.is_empty() {
            let section = self.section.name.clone().unwrap_or_default();
            return reader.fail(DocumentationError::EmptySection { section });
        }
        self.section.ended = true;
        Ok(())
    }

    fn end_comment(&mut self, reader: &Reader<'i>) -> Result<(), Fault<'i>> {
        self.end_section(reader)
    }

    /// QEMU's `_append_freeform`.
    fn freeform(&mut self, reader: &Reader<'i>, line: &str) -> Result<(), Fault<'i>> {
        let mut line = line.to_owned();
        if !at_least((4, 1, 0)) {
            let in_description = self.section.part == Part::Members;
            if in_description
                && self.section.after_blank()
                && line.chars().next().is_some_and(|c| !is_whitespace(c))
            {
                self.start_section(reader, None, None)?;
            }
            if in_description
                || !self
                    .section
                    .name
                    .as_ref()
                    .is_some_and(|name| name.starts_with("Example"))
            {
                line = line.trim_matches(is_whitespace).to_owned();
            }
        }
        if let Some(text) = freeform_description(&line) {
            return reader.fail(DocumentationError::NotInFreeform {
                text: text.to_owned(),
            });
        }
        if !at_least((2, 12, 0)) && self.section.part == Part::Members && line.contains("#optional")
        {
            return Err(fault(
                reader.comment,
                reader.comment,
                DocumentationError::OptionalTag,
            ));
        }
        self.section.append(reader, &line)
    }
}

/// The first documentation fault in `input`, a schema file, in the order
/// QEMU's parser finds them, or `None`. Reading stops at a malformed
/// expression, which QEMU reports first.
pub(crate) fn check(input: &str) -> Option<Fault<'_>> {
    if parser_version() < (2, 9, 0) {
        return None;
    }
    check_from(input).err()
}

/// A definition's documentation, read last, that no definition follows.
fn not_followed<'i>(pending: &Option<(&'i str, Option<String>)>) -> Result<(), Fault<'i>> {
    match pending {
        Some((at, Some(symbol))) => Err(fault(
            at,
            at,
            DocumentationError::NotFollowedByDefinition {
                symbol: symbol.clone(),
            },
        )),
        _ => Ok(()),
    }
}

fn check_from(mut rest: &str) -> Result<(), Fault<'_>> {
    // The documentation read last, while no expression has followed it.
    let mut pending = None;
    let mut skip_comments = true;
    loop {
        let (token, _) = next_token(rest, skip_comments);
        skip_comments = true;
        match token {
            Token::End(_) => return not_followed(&pending),
            Token::Comment(at, _) => {
                not_followed(&pending)?;
                let (symbol, after) = read(at, &mut ())?;
                pending = Some((at, symbol));
                rest = after;
                // 9.0 reads the token after a comment with comments included.
                skip_comments = parser_version() < (9, 0, 0) || at_least((9, 1, 0));
            }
            Token::Other(at) => {
                let Ok((Some(keys), after)) = super::common::top_level_expression(at) else {
                    return Ok(());
                };
                if keys
                    .iter()
                    .any(|key: &QapiString<'_>| key == "include" || key == "pragma")
                {
                    not_followed(&pending)?;
                } else if let Some((at, None)) = pending {
                    return Err(fault(
                        at,
                        at,
                        DocumentationError::DefinitionDocumentationRequired,
                    ));
                }
                pending = None;
                rest = after;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_schema_with_version;

    /// Each rule in the release that introduced it and, where one exists,
    /// the release before or after it, as QEMU's parser reports it.
    #[test]
    fn documentation_rules_follow_their_release() {
        for (version, input, expected) in [
            (
                (2, 8, 0),
                "##x\n# @Foo:\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (2, 9, 0),
                "##x\n# @Foo:\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((1, 1, "JunkAtStart")),
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n# @a: A.\n## x\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "JunkAtEnd")),
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n# @a: A.\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "Unterminated")),
            ),
            (
                (11, 1, 3),
                "##\n# @Foo:\n# @a: A.\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "Unterminated")),
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n#\t@a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((3, 1, "MissingSpace")),
            ),
            (
                (11, 1, 3),
                "##\n# @Foo:\n#\t@a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((3, 1, "MissingSpace")),
            ),
            (
                (2, 9, 0),
                "##\n# @Foo\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((2, 1, "SymbolWithoutColon")),
            ),
            (
                (9, 0, 0),
                "##\n# @Foo\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((2, 1, "SymbolWithoutColon")),
            ),
            (
                (2, 9, 0),
                "##\n# @:\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((2, 1, "EmptySymbol")),
            ),
            (
                (11, 1, 3),
                "##\n# @:\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((2, 1, "EmptySymbol")),
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n# @: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((3, 1, "EmptyName")),
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n# @: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((3, 1, "EmptyName")),
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n# @a: A.\n# @a: B.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "DuplicateName")),
            ),
            (
                (11, 1, 3),
                "##\n# @Foo:\n# @a: A.\n# @a: B.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "DuplicateName")),
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n# Since: 1.0\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "DescriptionAfterSection")),
            ),
            (
                (8, 1, 0),
                "##\n# @Foo:\n# Since: 1.0\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "DescriptionAfterSection")),
            ),
            (
                (11, 1, 3),
                "##\n# @Foo:\n# Since: 1.0\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "DescriptionAfterSection")),
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n# @a: A.\n#\n# Since: 1.0\n#\n# Since: 2.0\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((7, 1, "DuplicateSection")),
            ),
            (
                (11, 1, 3),
                "##\n# @Foo:\n# @a: A.\n#\n# Since: 1.0\n#\n# Since: 2.0\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((7, 1, "DuplicateSection")),
            ),
            (
                (8, 2, 0),
                "##\n# @foo:\n# @a: A.\n#\n# Errors: x\n#\n# Errors: y\n##\n{ 'command': 'foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (9, 0, 0),
                "##\n# @foo:\n# @a: A.\n#\n# Errors: x\n#\n# Errors: y\n##\n{ 'command': 'foo', 'data': { 'a': 'int' } }\n",
                Some((7, 1, "DuplicateSection")),
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n# @a: A.\n#\n# Since:\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((6, 1, "EmptySection")),
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n# @a: A.\n#\n# Since:\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((5, 1, "EmptySection")),
            ),
            (
                (2, 9, 0),
                "##\n# Text.\n# @a: x\n##\n",
                Some((3, 1, "NotInFreeform")),
            ),
            (
                (11, 1, 3),
                "##\n# Text.\n# @a: x\n##\n",
                Some((3, 1, "NotInFreeform")),
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n# @a: #optional A.\n##\n{ 'struct': 'Foo', 'data': { '*a': 'int' } }\n",
                Some((1, 1, "OptionalTag")),
            ),
            (
                (2, 11, 0),
                "##\n# @Foo:\n# @a: #optional A.\n##\n{ 'struct': 'Foo', 'data': { '*a': 'int' } }\n",
                Some((1, 1, "OptionalTag")),
            ),
            (
                (2, 12, 0),
                "##\n# @Foo:\n# @a: #optional A.\n##\n{ 'struct': 'Foo', 'data': { '*a': 'int' } }\n",
                None,
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n# @a: A.\n#\n# Features:\n# @f: F.\n#\n# Features:\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' }, 'features': [ 'f' ] }\n",
                Some((8, 1, "DuplicateFeatures")),
            ),
            (
                (8, 2, 0),
                "##\n# @Foo:\n# @a: A.\n#\n# Features:\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n# @a: A.\n#\n# Features:\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((6, 1, "NoFeatureDescriptions")),
            ),
            (
                (5, 1, 0),
                "##\n# @Foo:\n# @a: first\n#   second\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (5, 2, 0),
                "##\n# @Foo:\n# @a: first\n#   second\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "UnexpectedDeIndent")),
            ),
            (
                (8, 1, 0),
                "##\n# @Foo:\n# @a: first\n#   second\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (8, 0, 0),
                "##\n# @Foo:\n# @a:\n#     first\n#   second\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (8, 1, 0),
                "##\n# @Foo:\n# @a:\n#     first\n#   second\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((5, 1, "UnexpectedDeIndent")),
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n# @a:\n#     first\n#   second\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((5, 1, "UnexpectedDeIndent")),
            ),
            (
                (8, 2, 0),
                "##\n# @Foo:\n# @a:\n#     first\n#     second\n#   third\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((6, 1, "UnexpectedDeIndent")),
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n# @a:\n#     first\n#     second\n#   third\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (10, 2, 0),
                "##\n# @Foo:\n#   Intro.\n# More.\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (11, 1, 0),
                "##\n# @Foo:\n#   Intro.\n# More.\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "UnexpectedDeIndent")),
            ),
            (
                (5, 1, 0),
                "##\n# @Foo:\n#\n# = Heading\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (5, 2, 0),
                "##\n# @Foo:\n#\n# = Heading\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "HeadingInDefinition")),
            ),
            (
                (10, 0, 0),
                "##\n# @Foo:\n#\n# = Heading\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((4, 1, "HeadingInDefinition")),
            ),
            (
                (10, 1, 0),
                "##\n# @Foo:\n#\n# = Heading\n#\n# @a: A.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            ((8, 2, 0), "##\n# Text.\n#\n# = Heading\n##\n", None),
            (
                (9, 0, 0),
                "##\n# Text.\n#\n# = Heading\n##\n",
                Some((4, 1, "HeadingNotFirst")),
            ),
            ((10, 1, 0), "##\n# Text.\n#\n# = Heading\n##\n", None),
            (
                (9, 0, 0),
                "##\n# @Foo:\n# @a: A.\n#\n# Note: text\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (9, 1, 0),
                "##\n# @Foo:\n# @a: A.\n#\n# Note: text\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((5, 1, "UnsupportedSection")),
            ),
            (
                (10, 1, 0),
                "##\n# @Foo:\n# @a: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (10, 2, 0),
                "##\n# @Foo:\n# @a: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((3, 1, "LongLine")),
            ),
            (
                (10, 2, 0),
                "##\n# @Foo:\n# @a: A.\n#\n# ::\n#\n#   xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (10, 1, 0),
                "##\n# @Foo:\n# @a: One. Two.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (10, 2, 0),
                "##\n# @Foo:\n# @a: One. Two.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((3, 11, "SingleSpace")),
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n##\n{ 'include': 'x.json' }\n",
                Some((1, 1, "NotFollowedByDefinition")),
            ),
            (
                (2, 9, 0),
                "{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n##\n# @Bar:\n##\n",
                Some((2, 1, "NotFollowedByDefinition")),
            ),
            (
                (2, 8, 0),
                "##\n# Text.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (2, 9, 0),
                "##\n# Text.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((1, 1, "DefinitionDocumentationRequired")),
            ),
            (
                (11, 1, 3),
                "##\n# Text.\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((1, 1, "DefinitionDocumentationRequired")),
            ),
            (
                (8, 2, 0),
                "##\n# @Foo:\n# @a: A.\n##\n# plain\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (9, 0, 0),
                "##\n# @Foo:\n# @a: A.\n##\n# plain\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                Some((1, 1, "NotFollowedByDefinition")),
            ),
            (
                (9, 1, 0),
                "##\n# @Foo:\n# @a: A.\n##\n# plain\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (2, 9, 0),
                "##\n# @Foo:\n# @a: A.\n#\n# returns: text\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (11, 1, 3),
                "##\n# @Foo:\n# @a: A.\n#\n# returns: text\n##\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (2, 9, 0),
                "##\r\n# @Foo:\r\n#\r\n# @a: A.\r\n##\r\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
            (
                (11, 1, 3),
                "##\r\n# @Foo:\r\n#\r\n# @a: A.\r\n##\r\n{ 'struct': 'Foo', 'data': { 'a': 'int' } }\n",
                None,
            ),
        ] {
            let result = parse_schema_with_version(input, version);
            let actual = result.err().map(|error| {
                let rule = format!("{:?}", error.kind);
                let rule = rule
                    .trim_start_matches("Documentation(")
                    .split([' ', '(', ')'])
                    .next()
                    .unwrap_or_default()
                    .to_owned();
                (error.line, error.column, rule)
            });
            let expected = expected.map(|(line, column, rule)| (line, column, rule.to_owned()));
            assert_eq!(actual, expected, "{version:?} {input:?}");
        }
    }
}
