//! Shared nom parser helpers used across all parser versions.

use crate::ParseErrorKind;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_till, take_while, take_while1},
    character::complete::space0,
    combinator::{eof, not, opt, peek, recognize},
    error::{Error, ErrorKind},
    multi::{many1, separated_list0},
    sequence::{delimited, preceded},
};
use std::{borrow::Cow, collections::HashSet};

/// Decoded schema text; ordinary literals borrow their source.
pub(crate) type QapiString<'i> = Cow<'i, str>;

pub(crate) fn qcomment(input: &str) -> IResult<&str, &str> {
    delimited(
        (tag("#"), not(peek(tag("#"))), space0),
        not_line_ending,
        alt((line_ending, eof)),
    )
    .parse(input)
}

pub(crate) fn any_comment(input: &str) -> IResult<&str, &str> {
    delimited((tag("#"), space0), not_line_ending, alt((line_ending, eof)))
        .parse(input)
}

// QEMU's Python lexer uses str.isspace(), including the four C0 separators.
pub(crate) fn is_whitespace(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}')
}

/// QEMU reads schema files with Python's universal newline handling.
pub(crate) fn line_ending(input: &str) -> IResult<&str, &str> {
    alt((tag("\r\n"), tag("\n"), tag("\r"))).parse(input)
}

pub(crate) fn not_line_ending(input: &str) -> IResult<&str, &str> {
    take_till(|c| c == '\r' || c == '\n').parse(input)
}

pub(crate) fn lines(mut input: &str) -> impl Iterator<Item = &str> {
    std::iter::from_fn(move || {
        if input.is_empty() {
            return None;
        }
        let end = input.find(['\r', '\n']).unwrap_or(input.len());
        let line = &input[..end];
        input = &input[end..];
        if input.starts_with("\r\n") {
            input = &input[2..];
        } else if !input.is_empty() {
            input = &input[1..];
        }
        Some(line)
    })
}

pub(crate) fn whitespace0(input: &str) -> IResult<&str, &str> {
    take_while(is_whitespace).parse(input)
}

pub(crate) fn whitespace1(input: &str) -> IResult<&str, &str> {
    take_while1(is_whitespace).parse(input)
}

/// Python whitespace that stays on one physical schema line.
pub(crate) fn line_space0(input: &str) -> IResult<&str, &str> {
    take_while(|c| is_whitespace(c) && !matches!(c, '\r' | '\n')).parse(input)
}

pub(crate) fn clean_lines(input: &str) -> IResult<&str, &str> {
    let comment = if super::parser_version() < (2, 9, 0) {
        any_comment
    } else {
        qcomment
    };
    recognize((whitespace0, opt((many1((whitespace0, comment)), whitespace0))))
        .parse(input)
}

/// The text of an expression parsed from `start` up to `rest`, without the
/// whitespace and comments before it.
pub(crate) fn expression_text<'i>(start: &'i str, rest: &'i str) -> &'i str {
    let start = clean_lines(start).map_or(start, |(start, _)| start);
    &start[..start.len() - rest.len()]
}

pub(crate) fn qtag<'i>(
    t: &'static str,
) -> impl Parser<&'i str, Output = &'i str, Error = Error<&'i str>> {
    preceded(clean_lines, tag(t))
}

pub(crate) fn string_model(version: (u32, u32, u32)) -> u8 {
    if version < (2, 4, 0) {
        0
    } else if version < (4, 2, 0) {
        1
    } else {
        2
    }
}

fn string_error(input: &str) -> nom::Err<Error<&str>> {
    nom::Err::Error(Error::new(input, ErrorKind::Escaped))
}

/// Decode a literal, borrowing ordinary text and owning escaped values.
pub fn qstring(input: &str) -> IResult<&str, QapiString<'_>> {
    let (body, _) = qtag("'").parse(input)?;
    let version = super::parser_version();
    let model = string_model(version);
    let modern = model == 2;
    let mut offset = 0;
    let mut decoded: Option<String> = None;
    while offset < body.len() {
        let c = body[offset..].chars().next().unwrap();
        if c == '\'' {
            return Ok((
                &body[offset + 1..],
                decoded
                    .map_or_else(|| Cow::Borrowed(&body[..offset]), Cow::Owned),
            ));
        }
        if matches!(c, '\r' | '\n')
            || (modern && !c.is_ascii_graphic() && c != ' ')
        {
            return Err(string_error(&body[offset..]));
        }
        if c == '\\' {
            let value =
                decoded.get_or_insert_with(|| body[..offset].to_owned());
            offset += 1;
            let Some(escape) = body[offset..].chars().next() else {
                return Err(string_error(&body[offset..]));
            };
            if matches!(escape, '\r' | '\n') {
                return Err(string_error(&body[offset..]));
            }
            offset += escape.len_utf8();
            let c = if model == 0 {
                escape
            } else if !modern {
                match escape {
                    'b' => '\u{8}',
                    'f' => '\u{c}',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' | '/' | '\'' | '"' => escape,
                    'u' => {
                        let Some(hex) = body.get(offset..offset + 4) else {
                            return Err(string_error(&body[offset..]));
                        };
                        let value = hex
                            .bytes()
                            .all(|byte| byte.is_ascii_hexdigit())
                            .then(|| u8::from_str_radix(hex, 16).ok())
                            .flatten()
                            .filter(|value| *value != 0 && *value <= 0x7f)
                            .ok_or_else(|| string_error(&body[offset..]))?;
                        offset += 4;
                        value as char
                    }
                    _ => {
                        return Err(string_error(
                            &body[offset - escape.len_utf8()..],
                        ));
                    }
                }
            } else if escape == '\\' {
                '\\'
            } else {
                return Err(string_error(&body[offset - escape.len_utf8()..]));
            };
            value.push(c);
        } else {
            if let Some(value) = &mut decoded {
                value.push(c);
            }
            offset += c.len_utf8();
        }
    }
    Err(string_error(&body[offset..]))
}

/// `true` or `false`, which QEMU's lexer reads from 2.4: before, either is
/// a stray character.
pub(crate) fn qbool(input: &str) -> IResult<&str, &str> {
    let (rest, literal) = alt((qtag("true"), qtag("false"))).parse(input)?;
    if super::parser_version() < (2, 4, 0) {
        let at = input.trim_start_matches(is_whitespace);
        return Err(nom::Err::Failure(Error::new(at, ErrorKind::Char)));
    }
    Ok((rest, literal))
}

/// Modern QAPI uses JSON separators. Retain the permissive legacy grammar
/// only when parsing pre-1.6 schemas.
pub(crate) fn take_list<'i, I, O>(
    item_parser: I,
) -> impl Parser<&'i str, Output = Vec<O>, Error = Error<&'i str>>
where
    I: Parser<&'i str, Output = O, Error = Error<&'i str>>,
{
    delimited(qtag("["), separated_items(item_parser), qtag("]"))
}

/// Parse a dict, refusing a duplicate key from QEMU 2.0. Before, the last
/// value of a key replaces the earlier one, in the earlier one's place.
pub(crate) fn take_dict<'i, I, O>(
    item_parser: I,
) -> impl Parser<&'i str, Output = Vec<O>, Error = Error<&'i str>>
where
    I: Parser<&'i str, Output = O, Error = Error<&'i str>>,
{
    take_keyed_dict(item_parser).map(|(items, _)| items)
}

/// [`take_dict`], with the dict's keys in their order in the source.
pub(crate) fn take_keyed_dict<'i, I, O>(
    mut item_parser: I,
) -> impl Parser<
    &'i str,
    Output = (Vec<O>, Vec<QapiString<'i>>),
    Error = Error<&'i str>,
>
where
    I: Parser<&'i str, Output = O, Error = Error<&'i str>>,
{
    move |input| {
        let mut keys = HashSet::new();
        let mut entry = |input| {
            let (after_key, key) = qstring(input)?;
            if !keys.insert(key.clone()) && super::parser_version() >= (2, 0, 0)
            {
                // QEMU reports a duplicate key at its value. Without the
                // colon before one, no item parses, and QEMU's parser
                // reports the token after the key as for any other key.
                if let Ok((value, _)) = qtag(":").parse(after_key) {
                    return Err(nom::Err::Failure(Error::new(
                        value,
                        ErrorKind::Verify,
                    )));
                }
            }
            item_parser.parse(input).map(|(rest, item)| (rest, (key, item)))
        };
        let (rest, entries) =
            delimited(qtag("{"), separated_items(&mut entry), qtag("}"))
                .parse(input)?;
        if super::parser_version() >= (2, 0, 0) {
            return Ok((
                rest,
                entries.into_iter().map(|(key, item)| (item, key)).unzip(),
            ));
        }
        let mut items: Vec<(QapiString<'i>, O)> =
            Vec::with_capacity(entries.len());
        for (key, item) in entries {
            match items.iter_mut().find(|(earlier, _)| *earlier == key) {
                Some(earlier) => earlier.1 = item,
                None => items.push((key, item)),
            }
        }
        Ok((rest, items.into_iter().map(|(key, item)| (item, key)).unzip()))
    }
}

fn separated_items<'i, I, O>(
    mut item_parser: I,
) -> impl Parser<&'i str, Output = Vec<O>, Error = Error<&'i str>>
where
    I: Parser<&'i str, Output = O, Error = Error<&'i str>>,
{
    move |input| {
        // QEMU's parser requires the commas from 1.6.
        if super::parser_version() < (1, 6, 0) {
            separated_list_flexible(|i| item_parser.parse(i)).parse(input)
        } else {
            separated_list0(qtag(","), |i| item_parser.parse(i)).parse(input)
        }
    }
}

/// Parse items separated by optional commas. Accepts:
/// - `a, b, c` (standard)
/// - `a b c` (missing commas)
/// - `a, b, c,` (trailing comma)
fn separated_list_flexible<'i, I, O>(
    mut item_parser: I,
) -> impl Parser<&'i str, Output = Vec<O>, Error = Error<&'i str>>
where
    I: Parser<&'i str, Output = O, Error = Error<&'i str>>,
{
    move |mut input: &'i str| {
        let mut items = Vec::new();
        // Try to parse the first item (with optional leading whitespace via clean_lines)
        let first_input =
            clean_lines.parse(input).map(|(i, _)| i).unwrap_or(input);
        match item_parser.parse(first_input) {
            Ok((remaining, item)) => {
                items.push(item);
                input = remaining;
            }
            Err(nom::Err::Error(_)) => return Ok((input, items)),
            Err(e) => return Err(e),
        }
        // Parse subsequent items, each optionally preceded by a comma
        loop {
            // Consume optional comma
            let after_comma = match qtag::<'i>(",").parse(input) {
                Ok((remaining, _)) => remaining,
                Err(nom::Err::Error(_)) => input,
                Err(e) => return Err(e),
            };
            // Try to parse next item
            let after_ws = clean_lines
                .parse(after_comma)
                .map(|(i, _)| i)
                .unwrap_or(after_comma);
            match item_parser.parse(after_ws) {
                Ok((remaining, item)) => {
                    if remaining.len() == after_ws.len() {
                        return Err(nom::Err::Error(Error::new(
                            after_ws,
                            ErrorKind::Many0,
                        )));
                    }
                    items.push(item);
                    input = remaining;
                }
                Err(nom::Err::Error(_)) => {
                    // No more items — if we consumed a comma, that's a trailing comma (ok)
                    return Ok((after_comma, items));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

pub(crate) fn take_kv<'i, I, O>(
    key: &'static str,
    value_parser: I,
) -> impl Parser<&'i str, Output = O, Error = Error<&'i str>>
where
    I: Parser<&'i str, Output = O, Error = Error<&'i str>>,
{
    delimited(
        (
            nom::combinator::verify(qstring, move |name: &QapiString<'i>| {
                name.as_ref() == key
            }),
            qtag(":"),
        ),
        value_parser,
        clean_lines,
    )
}

/// A key of a dict that its parser does not read, with its value, which
/// is skipped. Before 2.4 the generators ignore such a key, whatever it
/// holds. From 2.4 it is a key outside `read`, which the validation
/// refuses with the keys that the release does not know; a key in `read`
/// whose value does not parse stays an error of the parser.
pub(crate) fn other_key<'i>(
    read: &'static [&'static str],
) -> impl Parser<&'i str, Output = (), Error = Error<&'i str>> {
    move |input: &'i str| {
        let (value, key) =
            (qstring, qtag(":")).map(|(key, _)| key).parse(input)?;
        if super::parser_version() >= (2, 4, 0) && read.contains(&key.as_ref())
        {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }
        let mut lexer = Lexer::new(value);
        lexer.advance().and_then(|()| lexer.value()).map_err(|(at, _)| {
            nom::Err::Error(Error::new(at, ErrorKind::Fail))
        })?;
        clean_lines.map(|_| ()).parse(lexer.token.1)
    }
}

/// A key of a definition that QEMU's generators read from `since`. Before
/// 2.4 an earlier release ignores it, so it is left to [`other_key`]. From
/// 2.4 the parser reads every key of a definition, and the validation
/// refuses those of later releases, so a key that 2.4 or a later release
/// brings is read `since` 2.4.
pub(crate) fn read_since<'i, I, O>(
    since: (u32, u32, u32),
    mut key_parser: I,
) -> impl Parser<&'i str, Output = O, Error = Error<&'i str>>
where
    I: Parser<&'i str, Output = O, Error = Error<&'i str>>,
{
    move |input| {
        let version = super::parser_version();
        if version < (2, 4, 0) && version < since {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }
        key_parser.parse(input)
    }
}

/// Before 1.6, where QEMU's parser fails ([`crate::ParserCrash`]) on
/// `input`, as [`legacy_schema`] reads it.
pub(crate) fn unreadable_line(
    input: &str,
) -> Option<(&str, crate::ParserCrash)> {
    legacy_schema(input).err().map(|(offset, crash)| (&input[offset..], crash))
}

/// From 1.2 to 1.5, `input` with each expression whose characters QEMU's
/// tokenizer drops, other than whitespace and comments without
/// punctuation, written as that parser reads it: its strings, objects and
/// arrays, each string quoted, on its first line, followed by its other
/// lines' ends. `None` when no expression has such characters, or the
/// parser fails on the input.
pub(crate) fn legacy_text(input: &str) -> Option<String> {
    let version = super::parser_version();
    if !((1, 2, 0)..(1, 6, 0)).contains(&version) {
        return None;
    }
    let expressions = legacy_schema(input).ok()?;
    let mut text = String::with_capacity(input.len());
    let mut copied = 0;
    for expression in expressions.iter().filter(|expression| expression.dropped)
    {
        text.push_str(&input[copied..expression.start]);
        expression.value.write(&mut text);
        let lines = &input[expression.start..expression.end];
        text.extend(lines.chars().filter(|c| matches!(c, '\r' | '\n')));
        copied = expression.end;
    }
    (copied > 0).then(|| text + &input[copied..])
}

/// A value as the parsers before 1.6 read it: every token is a string,
/// the punctuation too.
enum LegacyValue {
    String(String),
    Array(Vec<LegacyValue>),
    Object(Vec<(String, LegacyValue)>),
}

impl LegacyValue {
    fn write(&self, text: &mut String) {
        match self {
            Self::String(value) => {
                text.push('\'');
                for c in value.chars() {
                    if matches!(c, '\'' | '\\') {
                        text.push('\\');
                    }
                    text.push(c);
                }
                text.push('\'');
            }
            Self::Array(values) => {
                text.push('[');
                for (position, value) in values.iter().enumerate() {
                    if position > 0 {
                        text.push_str(", ");
                    }
                    value.write(text);
                }
                text.push(']');
            }
            Self::Object(entries) => {
                text.push('{');
                for (position, (key, value)) in entries.iter().enumerate() {
                    if position > 0 {
                        text.push_str(", ");
                    }
                    Self::String(key.clone()).write(text);
                    text.push_str(": ");
                    value.write(text);
                }
                text.push('}');
            }
        }
    }
}

/// An expression of a schema before 1.6: its lines, from byte `start` to
/// `end`, the object QEMU's parser reads from them, and whether its
/// tokenizer dropped a character that the grammar here would read.
struct LegacyExpression {
    start: usize,
    end: usize,
    value: LegacyValue,
    dropped: bool,
}

/// The expressions of a schema as QEMU's parser before 1.6 reads them, or
/// the byte offset at which it fails and why. It skips the lines that
/// start with `#` and the empty ones, continues an expression on each line
/// that starts with a space and evaluates it when a line starts otherwise.
/// Its tokenizer reads punctuation and strings, and skips spaces and
/// newlines; before 1.2 it never finishes at any other character, from 1.2
/// it drops them, letters such as those of `true` and `false` included.
/// The object parser takes every token as a value, so a missing one makes
/// it run out of tokens.
fn legacy_schema(
    input: &str,
) -> Result<Vec<LegacyExpression>, (usize, crate::ParserCrash)> {
    use crate::ParserCrash;
    let version = super::parser_version();
    if version >= (1, 6, 0) {
        return Ok(Vec::new());
    }
    // The characters of the expression, each with its offset in `input`.
    let mut expression: Vec<(usize, char)> = Vec::new();
    let mut start = 0;
    let mut end = 0;
    let mut expressions = Vec::new();
    let mut evaluate = |expression: &[(usize, char)],
                        start,
                        end,
                        next: Option<usize>| {
        let (tokens, dropped) = legacy_tokens(expression, version)?;
        let fail = || {
            // An open expression that a line at the first column cut.
            let depth = tokens.iter().fold(
                0isize,
                |depth, (token, punctuation)| match (
                    punctuation,
                    token.as_str(),
                ) {
                    (true, "{" | "[") => depth + 1,
                    (true, "}" | "]") => depth - 1,
                    _ => depth,
                },
            );
            match next {
                Some(next) if depth > 0 => (next, ParserCrash::UnindentedLine),
                _ => (start, ParserCrash::IncompleteExpression),
            }
        };
        let mut position = 0;
        let value = legacy_value(&tokens, &mut position).ok_or_else(fail)?;
        if !matches!(value, LegacyValue::Object(_)) {
            return Err((start, ParserCrash::NotAnObject));
        }
        expressions.push(LegacyExpression { start, end, value, dropped });
        Ok(())
    };
    let mut offset = 0;
    for line in input.split_inclusive(['\r', '\n']) {
        let at = offset;
        offset += line.len();
        // Universal newlines: "\r\n" is one line end.
        if line == "\n" && input[..at].ends_with('\r') {
            end = offset;
            continue;
        }
        let blank = matches!(line, "\n" | "\r");
        if line.starts_with('#') || blank {
            continue;
        }
        if !line.starts_with(' ') && !expression.is_empty() {
            evaluate(&expression, start, end, Some(at))?;
            expression.clear();
        }
        if expression.is_empty() {
            start = at;
        }
        expression.extend(
            line.char_indices().map(|(index, c)| {
                (at + index, if c == '\r' { '\n' } else { c })
            }),
        );
        end = offset;
    }
    if !expression.is_empty() {
        evaluate(&expression, start, end, None)?;
    }
    Ok(expressions)
}

/// The tokens of an expression before 1.6, each with whether it is
/// punctuation, and whether the tokenizer dropped a character that the
/// grammar here would read: anything but whitespace and a comment without
/// punctuation or quotes. Before 1.2 a string ends at the next quote, from
/// 1.2 a backslash escapes the character after it.
#[allow(clippy::type_complexity)]
fn legacy_tokens(
    expression: &[(usize, char)],
    version: (u32, u32, u32),
) -> Result<(Vec<(String, bool)>, bool), (usize, crate::ParserCrash)> {
    use crate::ParserCrash;
    let mut tokens = Vec::new();
    let mut dropped = false;
    let mut comment = false;
    let mut chars = expression.iter().copied();
    while let Some((at, c)) = chars.next() {
        match c {
            '{' | '}' | ':' | ',' | '[' | ']' => {
                dropped |= comment;
                tokens.push((c.to_string(), true));
            }
            ' ' | '\n' => comment &= c != '\n',
            '\'' => {
                dropped |= comment;
                let mut value = String::new();
                loop {
                    let Some((_, c)) = chars.next() else {
                        return Err((at, ParserCrash::UnterminatedString));
                    };
                    match c {
                        '\'' => break,
                        '\\' if version >= (1, 2, 0) => match chars.next() {
                            Some((_, c)) => value.push(c),
                            None => {
                                return Err((
                                    at,
                                    ParserCrash::UnterminatedString,
                                ));
                            }
                        },
                        c => value.push(c),
                    }
                }
                tokens.push((value, false));
            }
            _ if version < (1, 2, 0) => {
                return Err((at, ParserCrash::StrayCharacter));
            }
            '#' => comment = true,
            c => dropped |= !comment && !is_whitespace(c),
        }
    }
    Ok((tokens, dropped))
}

/// QEMU's object parser before 1.6 at `tokens[*position]`: `None` where it
/// runs out of tokens.
fn legacy_value(
    tokens: &[(String, bool)],
    position: &mut usize,
) -> Option<LegacyValue> {
    let token =
        |position: usize| tokens.get(position).map(|(token, _)| token.as_str());
    let (first, _) = tokens.get(*position)?;
    *position += 1;
    let close = match first.as_str() {
        "{" => "}",
        "[" => "]",
        _ => return Some(LegacyValue::String(first.clone())),
    };
    let mut object: Vec<(String, LegacyValue)> = Vec::new();
    let mut array = Vec::new();
    while token(*position)? != close {
        let key = (close == "}").then(|| {
            // The key, and the token after it, which should be the colon.
            *position += 2;
            tokens[*position - 2].0.clone()
        });
        let value = legacy_value(tokens, position)?;
        if token(*position)? == "," {
            *position += 1;
        }
        match key {
            // A later value of a key replaces the earlier one in place.
            Some(key) => {
                match object.iter_mut().find(|(earlier, _)| *earlier == key) {
                    Some(earlier) => earlier.1 = value,
                    None => object.push((key, value)),
                }
            }
            None => array.push(value),
        }
    }
    *position += 1;
    Some(match close {
        "}" => LegacyValue::Object(object),
        _ => LegacyValue::Array(array),
    })
}

/// Where QEMU's parser rejects the top-level expression that starts `input`
/// as malformed, before any definition's grammar applies: the input from
/// the token it stops at, and the kind of fault. `None` when the expression
/// is well formed, or for a release before 1.6, whose parser reported no
/// position.
pub(crate) fn syntax_error(input: &str) -> Option<Fault<'_>> {
    if super::parser_version() < (1, 6, 0) {
        return None;
    }
    let mut lexer = Lexer::new(input);
    lexer.advance().and_then(|()| lexer.value()).err()
}

/// A well-formed top-level expression that starts `input`, as QEMU's parser
/// reads it: the keys of its object (`None` for another value) and the
/// input from the token after it. `Err` holds where the expression stops
/// being well formed, as [`syntax_error`] reports it.
#[allow(clippy::type_complexity)]
pub(crate) fn top_level_expression(
    input: &str,
) -> Result<(Option<Vec<QapiString<'_>>>, &str), Fault<'_>> {
    let mut lexer = Lexer::new(input);
    lexer.advance()?;
    let value = lexer.value()?;
    let keys = match value.value {
        Raw::Object(entries) => {
            Some(entries.into_iter().map(|(key, _)| key).collect())
        }
        _ => None,
    };
    Ok((keys, lexer.token.1))
}

/// The well-formed expression that starts `input`, as QEMU's parser from
/// 1.6 reads it.
pub(crate) fn expression_value(input: &str) -> Option<RawValue<'_>> {
    let mut lexer = Lexer::new(input);
    lexer.advance().ok()?;
    lexer.value().ok()
}

/// Where QEMU's lexer rejects the token that starts `input`: a stray
/// character or a malformed string.
pub(crate) fn token_error(input: &str) -> Option<Fault<'_>> {
    Lexer::new(input).advance().err()
}

/// The input from the token that QEMU's lexer reads next in `input`, after
/// space and comments, or from where it rejects one. The end of the input
/// is the newline that ends the file.
pub(crate) fn next_token(input: &str) -> &str {
    let mut lexer = Lexer::new(input);
    match lexer.advance() {
        Ok(()) => lexer.token.1,
        Err((at, _)) => at,
    }
}

#[derive(PartialEq, Eq)]
enum Token<'i> {
    Punct(char),
    String(QapiString<'i>),
    Literal(Raw<'i>),
    Comment,
    End,
}

/// A JSON value as the release's parser reads it, with its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawValue<'i> {
    /// The value in its schema file.
    pub text: &'i str,
    /// What it is.
    pub value: Raw<'i>,
}

/// The kinds of [`RawValue`]: QEMU's parser reads `true` and `false` from
/// 2.4 and `null` from 2.4 to 4.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Raw<'i> {
    String(QapiString<'i>),
    Bool(bool),
    Null,
    Array(Vec<RawValue<'i>>),
    Object(Vec<(QapiString<'i>, RawValue<'i>)>),
}

impl<'i> RawValue<'i> {
    /// Whether Python takes the value for true.
    pub(crate) fn truthy(&self) -> bool {
        match &self.value {
            Raw::String(value) => !value.is_empty(),
            Raw::Bool(value) => *value,
            Raw::Null => false,
            Raw::Array(values) => !values.is_empty(),
            Raw::Object(entries) => !entries.is_empty(),
        }
    }
}

/// The key/value entries of a dict that the typed grammar refused, kept
/// for the validation.
pub(crate) type Malformed<'i> = Vec<(QapiString<'i>, RawValue<'i>)>;

/// The value of `key` among `malformed` ones.
pub(crate) fn malformed<'t, 'i>(
    malformed: &'t Malformed<'i>,
    key: &str,
) -> Option<&'t RawValue<'i>> {
    malformed
        .iter()
        .find(|(name, _)| name.as_ref() == key)
        .map(|(_, value)| value)
}

/// A JSON value of any shape the release's parser reads, from 2.4, as a
/// [`RawValue`]: what a grammar that refuses its shape keeps for the
/// validation, which refuses it where QEMU does.
pub(crate) fn raw_value(input: &str) -> IResult<&str, RawValue<'_>> {
    if super::parser_version() < (2, 4, 0) {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    let mut lexer = Lexer::new(input);
    let value = lexer
        .advance()
        .and_then(|()| lexer.value())
        .map_err(|(at, _)| nom::Err::Error(Error::new(at, ErrorKind::Fail)))?;
    Ok((lexer.token.1, value))
}

/// Whether a definition has its name key `meta`: from 2.4 of any value,
/// which the validation checks, before 2.4 a name that is not empty.
pub(crate) fn named(keys: &[QapiString<'_>], meta: &str, name: &str) -> bool {
    if super::parser_version() >= (2, 4, 0) {
        keys.iter().any(|key| key == meta)
    } else {
        !name.is_empty()
    }
}

/// Keep a malformed entry, except `null` for a key of `absent`, which QEMU
/// 2.4 to 4.1 take for an absent one: they read the key with `get`.
pub(crate) fn keep<'i>(
    malformed: &mut Malformed<'i>,
    absent: &[&str],
    (key, value): (QapiString<'i>, RawValue<'i>),
) {
    if value.value != Raw::Null || !absent.contains(&key.as_ref()) {
        malformed.push((key, value));
    }
}

/// From 2.4, a key of `read` whose value the key's own grammar refused,
/// with that value, which the validation refuses where QEMU does.
pub(crate) fn malformed_key<'i>(
    read: &'static [&'static str],
) -> impl Parser<
    &'i str,
    Output = (QapiString<'i>, RawValue<'i>),
    Error = Error<&'i str>,
> {
    move |input: &'i str| {
        let (value, key) =
            (qstring, qtag(":")).map(|(key, _)| key).parse(input)?;
        if !read.contains(&key.as_ref()) {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }
        let (rest, value) = raw_value(value)?;
        let (rest, _) = clean_lines(rest)?;
        Ok((rest, (key, value)))
    }
}

/// QEMU's tokenizer and expression parser.
struct Lexer<'i> {
    /// The input after the current token.
    rest: &'i str,
    /// The current token and the input from its start.
    token: (Token<'i>, &'i str),
}

/// Where QEMU's parser stops on a malformed expression, and why.
pub(crate) type Fault<'i> = (&'i str, ParseErrorKind);

impl<'i> Lexer<'i> {
    fn new(input: &'i str) -> Self {
        Self { rest: input, token: (Token::End, input) }
    }

    fn advance(&mut self) -> Result<(), Fault<'i>> {
        loop {
            let start = self.rest.trim_start_matches(is_whitespace);
            let Some(c) = start.chars().next() else {
                // QEMU's end of input is the newline that ends the file.
                let end = self.rest.strip_suffix('\n').unwrap_or(self.rest);
                let end = end.strip_suffix('\r').unwrap_or(end);
                self.token = (Token::End, &self.rest[end.len()..]);
                self.rest = start;
                return Ok(());
            };
            if c == '#' {
                self.rest =
                    start.trim_start_matches(|c| c != '\r' && c != '\n');
                // From 2.9 a documentation comment ends the expression.
                if start.starts_with("##")
                    && super::parser_version() >= (2, 9, 0)
                {
                    self.token = (Token::Comment, start);
                    return Ok(());
                }
                continue;
            }
            let (rest, token) = if "{}:,[]".contains(c) {
                (&start[1..], Token::Punct(c))
            } else if c == '\'' {
                let (rest, value) = qstring(start)
                    .map_err(|_| (start, ParseErrorKind::MalformedString))?;
                (rest, Token::String(value))
            } else if let Some((word, value)) =
                [("true", Raw::Bool(true)), ("false", Raw::Bool(false))]
                    .into_iter()
                    .chain(
                        (super::parser_version() < (4, 2, 0))
                            .then_some(("null", Raw::Null)),
                    )
                    .filter(|_| super::parser_version() >= (2, 4, 0))
                    .find(|(word, _)| start.starts_with(word))
            {
                (&start[word.len()..], Token::Literal(value))
            } else {
                return Err((start, ParseErrorKind::UnexpectedCharacter));
            };
            self.rest = rest;
            self.token = (token, start);
            return Ok(());
        }
    }

    /// The fault of a current token that is out of its place.
    fn fault<T>(&self) -> Result<T, Fault<'i>> {
        Err(match self.token.0 {
            Token::End => (self.token.1, ParseErrorKind::UnexpectedEnd),
            _ => (self.token.1, ParseErrorKind::UnexpectedCharacter),
        })
    }

    /// The value that starts at the current token; the token after it
    /// becomes current.
    fn value(&mut self) -> Result<RawValue<'i>, Fault<'i>> {
        let start = self.token.1;
        let value = match std::mem::replace(&mut self.token.0, Token::End) {
            Token::Punct('{') => {
                self.advance()?;
                Raw::Object(self.members()?)
            }
            Token::Punct('[') => {
                self.advance()?;
                Raw::Array(self.values()?)
            }
            Token::String(value) => {
                self.advance()?;
                Raw::String(value)
            }
            Token::Literal(value) => {
                self.advance()?;
                value
            }
            token => {
                self.token.0 = token;
                return self.fault();
            }
        };
        let text = &start[..start.len() - self.token.1.len()];
        Ok(RawValue { text: text.trim_end_matches(is_whitespace), value })
    }

    fn members(
        &mut self,
    ) -> Result<Vec<(QapiString<'i>, RawValue<'i>)>, Fault<'i>> {
        let mut entries = Vec::new();
        if self.token.0 == Token::Punct('}') {
            self.advance()?;
            return Ok(entries);
        }
        let mut keys = HashSet::new();
        loop {
            let key = match std::mem::replace(&mut self.token.0, Token::End) {
                Token::String(key) => key,
                token => {
                    self.token.0 = token;
                    return self.fault();
                }
            };
            self.advance()?;
            if self.token.0 != Token::Punct(':') {
                return self.fault();
            }
            self.advance()?;
            if !keys.insert(key.clone()) && super::parser_version() >= (2, 0, 0)
            {
                return Err((self.token.1, ParseErrorKind::DuplicateKey));
            }
            entries.push((key, self.value()?));
            match self.token.0 {
                Token::Punct('}') => {
                    self.advance()?;
                    return Ok(entries);
                }
                Token::Punct(',') => self.advance()?,
                _ => return self.fault(),
            }
        }
    }

    fn values(&mut self) -> Result<Vec<RawValue<'i>>, Fault<'i>> {
        let mut values = Vec::new();
        if self.token.0 == Token::Punct(']') {
            self.advance()?;
            return Ok(values);
        }
        loop {
            values.push(self.value()?);
            match self.token.0 {
                Token::Punct(']') => {
                    self.advance()?;
                    return Ok(values);
                }
                Token::Punct(',') => self.advance()?,
                _ => return self.fault(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ParseError, ParseErrorKind, ParserCrash, parse_schema,
        set_parser_version,
    };

    /// What the release's parser fails on, if the error is its crash.
    fn crash(error: ParseError) -> Option<ParserCrash> {
        match error.kind {
            ParseErrorKind::Crash(crash) => Some(crash),
            _ => None,
        }
    }

    #[test]
    fn strings_follow_qemu_lexical_rules() {
        set_parser_version((11, 1, 3));
        use super::qstring;
        for input in ["''", "'name with spaces'", r"'path\\name'"] {
            assert_eq!(qstring(input).unwrap().0, "");
        }
        for input in [
            "'line\nbreak'",
            "'tab\there'",
            "'snowman ☃'",
            r"'bad\n'",
            r"'bad\'quote'",
        ] {
            assert!(qstring(input).is_err(), "accepted {input}");
        }
    }

    #[test]
    fn modern_separators_match_qemu() {
        set_parser_version((11, 1, 3));
        for input in [
            "{ 'enum': 'E', 'data': ['a' 'b'] }",
            "{ 'enum': 'E', 'data': ['a',] }",
            "{ 'enum': 'E' 'data': ['a'] }",
            "{ 'enum': 'E', 'data': ['a'], }",
            "{ 'struct': 'S', 'data': { 'x': 'str', 'x': 'int' } }",
            "{ 'enum': 'E', 'enum': 'F', 'data': [] }",
        ] {
            assert!(
                parse_schema(input).is_err(),
                "Accepted invalid QAPI: {input}"
            );
        }
        parse_schema("{ 'enum': 'E', 'data': ['a', 'b'] }").unwrap();
    }

    #[test]
    fn dictionary_keywords_are_exact_strings() {
        set_parser_version((11, 1, 3));
        assert!(
            parse_schema("{ 'enum# comment\n': 'E', 'data': ['a'] }").is_err()
        );
        // An expression without a metatype is kept for the validation.
        let schema = parse_schema("{ 'enum ': 'E', 'data': ['a'] }").unwrap();
        assert!(matches!(schema.0[0], crate::QapiSchemaToken::NoMetatype(_)));
        // A key that is no definition's is kept for the validation.
        let schema = parse_schema("{ 'enum': 'E', 'data ': ['a'] }").unwrap();
        assert!(
            matches!(&schema.0[0], crate::QapiSchemaToken::Enum(value) if value.keys == ["enum", "data "])
        );
        parse_schema("{ 'enum' # comment\n : 'E', 'data': ['a'] }").unwrap();
    }

    #[test]
    fn malformed_expressions_are_reported_where_qemus_parser_stops() {
        use crate::ParseErrorKind::*;
        // Positions as QEMU 4.1 and 11.1 report them.
        for (version, input, position, kind) in [
            (
                (4, 1, 0),
                "{ 'enum': 'E', 'data': [ 'a', ] }",
                (1, 31),
                UnexpectedCharacter,
            ),
            (
                (11, 1, 3),
                "{ 'enum': 'E',\n  'enum': 'F' }",
                (2, 11),
                DuplicateKey,
            ),
            (
                (11, 1, 3),
                r"{ 'enum': 'E\x', 'data': [] }",
                (1, 11),
                MalformedString,
            ),
            (
                (11, 1, 3),
                "{ 'enum': 'E', 'data': [ 1 ] }",
                (1, 26),
                UnexpectedCharacter,
            ),
            ((11, 1, 3), "{ 'enum': 'E', }", (1, 16), UnexpectedCharacter),
            ((11, 1, 3), "{ 'enum': 'E',\n", (1, 15), UnexpectedEnd),
            ((11, 1, 3), "{ 'enum': 'E',", (1, 15), UnexpectedEnd),
            (
                (11, 1, 3),
                "{ 'enum': 'E', 'data': null }",
                (1, 24),
                UnexpectedCharacter,
            ),
            (
                (11, 1, 3),
                "{ 'enum': 'F'\n  'data': [] }",
                (2, 3),
                UnexpectedCharacter,
            ),
            // Well formed, so only the grammar of a directive rejects it.
            (
                (11, 1, 3),
                "{ 'pragma': { 'doc-required': 'x' } }",
                (1, 1),
                NoDefinition,
            ),
            // Before 1.6 QEMU's parser reported no position.
            (
                (2, 3, 0),
                "{ 'enum': 'E', 'data': [ 1 ] }",
                (1, 26),
                UnexpectedCharacter,
            ),
            ((1, 5, 0), "{ 'enum': 'E', 'data': [ 1 ] }", (1, 1), NoDefinition),
        ] {
            set_parser_version(version);
            let error = parse_schema(input).unwrap_err();
            assert_eq!(
                ((error.line, error.column), error.kind),
                (position, kind),
                "{version:?} {input}"
            );
        }
        // A definition keeps a value of another shape for the validation.
        for (version, input) in [
            ((11, 1, 3), "{ 'enum': 'E', 'data': 'x' }"),
            ((4, 1, 0), "{ 'enum': 'E', 'data': null }"),
        ] {
            set_parser_version(version);
            parse_schema(input).unwrap();
        }
        // QEMU reads null until 4.1.
        set_parser_version((4, 2, 0));
        let error = parse_schema("{ 'enum': 'E', 'data': null }").unwrap_err();
        assert_eq!((error.line, error.column), (1, 24));
    }

    #[test]
    fn double_hash_lines_end_an_expression_from_2_9() {
        let input = "{'struct':'Ss',\n## x\n'data':{}}";
        set_parser_version((2, 8, 0));
        parse_schema(input).unwrap();
        set_parser_version((2, 9, 0));
        let error = parse_schema(input).unwrap_err();
        assert_eq!((error.line, error.column), (2, 1));
    }

    #[test]
    fn comments_that_do_not_start_their_line_crash_qemu_before_1_6() {
        let indented = "  ##\n  # @Foo:\n  ##\n{ 'type': 'Foo', 'data': {} }";
        let trailing = "{ 'type': 'Foo', 'data': {} } # plain words";
        for (version, input, crashes) in [
            ((1, 1, 0), indented, true),
            ((1, 1, 0), trailing, true),
            ((1, 2, 0), indented, true),
            ((1, 5, 0), "{ 'type': 'Foo', 'data': {} } # see: Bar", false),
            ((1, 5, 0), "{ 'type': 'Foo',\n  # see: Bar\n  'data': {} }", true),
            ((1, 5, 0), trailing, false),
            ((1, 5, 0), "{ 'type': 'Foo', 'data': { 'a': 'x#y' } }", false),
            ((1, 6, 0), indented, false),
        ] {
            set_parser_version(version);
            let crash = parse_schema(input).err().and_then(crash);
            assert_eq!(crash.is_some(), crashes, "{version:?} {input}");
        }
        set_parser_version((1, 1, 0));
        let error = parse_schema(indented).unwrap_err();
        assert_eq!((error.line, error.column), (1, 3));
        // A line of an expression at the first column starts the next one.
        let unindented = "{ 'type': 'Foo',\n## x\n'data': {} }";
        for (version, crashes) in [((1, 5, 0), true), ((1, 6, 0), false)] {
            set_parser_version(version);
            let crash = parse_schema(unindented).err().and_then(crash);
            assert_eq!(crash.is_some(), crashes, "{version:?}");
        }
        set_parser_version((1, 5, 0));
        let error = parse_schema(unindented).unwrap_err();
        assert_eq!(
            (error.line, error.column, crash(error)),
            (3, 1, Some(crate::ParserCrash::UnindentedLine))
        );
    }

    #[test]
    fn parsers_before_1_6_drop_or_stop_at_other_characters() {
        use crate::ParserCrash::*;
        let flag = "{ 'command': 'cmd', 'gen': false,\n  'data': { 'a': 'str' } }\n{ 'enum': 'E', 'data': [] }\n";
        let missing = "{ 'command': 'cmd', 'boxed': false }";
        for (version, input, crash) in [
            ((1, 1, 0), flag, Some(StrayCharacter)),
            ((1, 2, 0), flag, None),
            ((1, 1, 0), missing, Some(StrayCharacter)),
            ((1, 5, 0), missing, Some(IncompleteExpression)),
            ((1, 3, 0), "'cmd'\n", Some(NotAnObject)),
            (
                (1, 0, 0),
                "{ 'enum': 'E', 'data': [ 'a ] }",
                Some(UnterminatedString),
            ),
            (
                (1, 3, 0),
                "{ 'enum': 'E', 'data': [ 'a\\' ] }",
                Some(UnterminatedString),
            ),
            (
                (1, 1, 0),
                "{ 'enum': 'E',\t'data': [ 'a' ] }",
                Some(StrayCharacter),
            ),
            ((1, 3, 0), "{ 'enum': 'E',\t'data': [ 'a' ] }", None),
            // A quoted brace is a brace to those parsers.
            (
                (1, 3, 0),
                "{ 'type': 'S', 'data': { 'a': '{' } }",
                Some(IncompleteExpression),
            ),
            ((1, 6, 0), missing, None),
        ] {
            set_parser_version(version);
            let found = parse_schema(input).err().and_then(self::crash);
            assert_eq!(found, crash, "{version:?} {input}");
        }
        // From 1.2 the letters go, so 'gen' takes the comma for its value.
        set_parser_version((1, 2, 0));
        let text = super::legacy_text(flag).unwrap();
        assert_eq!(
            text,
            "{'command': 'cmd', 'gen': ',', 'data': {'a': 'str'}}\n\n{ 'enum': 'E', 'data': [] }\n"
        );
        parse_schema(&text).unwrap();
        assert_eq!(
            super::legacy_text("{ 'enum': 'E', # values\n  'data': [] }"),
            None
        );
        set_parser_version((11, 1, 3));
    }

    #[test]
    fn separators_and_duplicate_keys_follow_each_release() {
        // Commas from 1.6.
        let commas = "{ 'enum': 'E' 'data': ['a'] }";
        set_parser_version((1, 5, 0));
        parse_schema(commas).unwrap();
        set_parser_version((1, 6, 0));
        parse_schema(commas).unwrap_err();
        // Duplicate keys refused from 2.0, in a definition and in an
        // expression that no definition reads.
        for input in [
            "{ 'enum': 'E', 'data': ['a'], 'data': ['b'] }",
            "{ 'other': 'x', 'other': 'y' }",
        ] {
            set_parser_version((1, 7, 0));
            parse_schema(input).unwrap();
            set_parser_version((2, 0, 0));
            let error = parse_schema(input).unwrap_err();
            assert_eq!(
                error.kind,
                crate::ParseErrorKind::DuplicateKey,
                "{input}"
            );
        }
        set_parser_version((11, 1, 3));
    }

    #[test]
    fn historical_separator_compatibility() {
        set_parser_version((1, 0, 0));
        parse_schema("{ 'enum': 'E' 'data': ['a' 'b',], }").unwrap();
    }

    #[test]
    fn comments_can_end_at_eof() {
        parse_schema("# final comment").unwrap();
        parse_schema("{ 'enum': 'E', 'data': ['a'] } # final comment").unwrap();
    }
    #[test]
    fn string_decoding_follows_legacy_escape_eras() {
        use super::{QapiString, qstring};
        for (version, input, expected) in [
            ((2, 3, 0), r"'a\nb'", "anb"),
            ((2, 3, 0), r"'a\zb'", "azb"),
            ((2, 4, 0), r"'a\nb'", "a\nb"),
            ((4, 1, 0), r#"'\u0041\t\'\"\/\\'"#, "A\t'\"/\\"),
            ((4, 1, 0), "'snowman ☃'", "snowman ☃"),
            ((4, 2, 0), r"'a\\b'", "a\\b"),
        ] {
            set_parser_version(version);
            assert_eq!(
                qstring(input).unwrap().1,
                expected,
                "{version:?} {input}"
            );
        }
        set_parser_version((4, 1, 0));
        for input in [r"'\u0000'", r"'\u0080'", r"'\u+041'", r"'\z'"] {
            assert!(qstring(input).is_err(), "{input}");
        }
        set_parser_version((11, 1, 3));
        assert!(matches!(
            qstring("'ordinary'").unwrap().1,
            QapiString::Borrowed("ordinary")
        ));
        assert!(matches!(qstring(r"'a\\b'").unwrap().1, QapiString::Owned(_)));
    }
}
