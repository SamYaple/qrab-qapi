//! Rendering of the IR as Rust tokens, with rustdoc and `#[qapi(...)]` attributes.

use super::{
    Attribute, Enum, EnumVariantKind, Struct, rustify_field, rustify_name, rustify_type,
    rustify_variant,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Enum {
    pub(crate) fn generate(&self) -> TokenStream {
        let enum_name = format_ident!("{}", rustify_name(&self.name));

        let enum_attrs = self.meta.attributes.iter().map(generate_attribute);
        let enum_doc = self.meta.doc.as_deref().map(doc_to_tokens);
        // Union branches hold their QAPI payload structs by value; boxing the
        // large ones would change the public variant types.
        let size_allow = self
            .meta
            .attributes
            .iter()
            .any(|attr| matches!(attr, Attribute::Unit(name) if name == "union_branch"))
            .then(|| {
                quote! {
                    #[allow(
                        clippy::large_enum_variant,
                        reason = "QAPI union branches hold their payload structs by value"
                    )]
                }
            });

        let variants = self.variants.iter().map(|variant| {
            let variant_name = format_ident!("{}", rustify_variant(&variant.name));

            let variant_attrs = variant.meta.attributes.iter().map(generate_attribute);
            let variant_doc = variant.meta.doc.as_deref().map(doc_to_tokens);

            match &variant.kind {
                EnumVariantKind::Unit => {
                    quote! {
                        #variant_doc
                        #(#variant_attrs)*
                        #variant_name,
                    }
                }
                EnumVariantKind::Tuple(type_name) => {
                    let inner: TokenStream = rustify_type(type_name).parse().unwrap();
                    let type_tokens = if variant.array {
                        quote! { Vec<#inner> }
                    } else {
                        inner
                    };
                    quote! {
                        #variant_doc
                        #(#variant_attrs)*
                        #variant_name(#type_tokens),
                    }
                }
            }
        });

        quote! {
            #enum_doc
            #(#enum_attrs)*
            #size_allow
            pub enum #enum_name {
                #(#variants)*
            }
        }
    }
}

impl Struct {
    pub(crate) fn generate(&self) -> TokenStream {
        let struct_name = format_ident!("{}", rustify_name(&self.name));
        let struct_attrs = self.meta.attributes.iter().map(generate_attribute);
        let struct_doc = self.meta.doc.as_deref().map(doc_to_tokens);

        let fields = self.fields.iter().map(|field| {
            let field_name: TokenStream = rustify_field(&field.name).parse().unwrap();
            let mut field_type: TokenStream = rustify_type(&field.r#type).parse().unwrap();
            if field.array {
                field_type = quote!( Vec<#field_type> );
            }
            // "boxed" marks a recursive field; it is internal, not a #[qapi] option.
            let is_boxed =
                |attr: &Attribute| matches!(attr, Attribute::Unit(name) if name == "boxed");
            if field.meta.attributes.iter().any(is_boxed) {
                field_type = quote!( Box<#field_type> );
            }
            if field.optional {
                field_type = quote!( Option<#field_type> );
            }

            let field_attrs = field
                .meta
                .attributes
                .iter()
                .filter(|attr| !is_boxed(attr))
                .map(generate_attribute);
            let field_doc = field.meta.doc.as_deref().map(doc_to_tokens);

            quote! {
                #field_doc
                #(#field_attrs)*
                pub #field_name: #field_type,
            }
        });

        quote! {
            #struct_doc
            #(#struct_attrs)*
            pub struct #struct_name {
                #(#fields)*
            }
        }
    }
}

fn generate_attribute(attribute: &Attribute) -> TokenStream {
    let options = generate_options(attribute);
    quote! {
        #[qapi( #(#options),* )]
    }
}

fn generate_options(attribute: &Attribute) -> Vec<TokenStream> {
    match attribute {
        Attribute::Item { name, value } => {
            let name = format_ident!("{}", name);
            vec![quote! { #name = #value }]
        }
        Attribute::Unit(name) => {
            let name = format_ident!("{}", name);
            vec![quote! { #name }]
        }
        Attribute::List(attributes) => attributes.iter().flat_map(generate_options).collect(),
        Attribute::Nested { name, inner } => {
            let name = format_ident!("{}", name);
            let inner_opts: Vec<TokenStream> = inner.iter().flat_map(generate_options).collect();
            vec![quote! { #name(#(#inner_opts),*) }]
        }
    }
}

/// Convert schema documentation into `#[doc = "..."]` attributes, one per
/// line, which prettyplease prints as `///` comments.
fn doc_to_tokens(doc: &str) -> TokenStream {
    let lines = render_doc(doc).into_iter().map(|line| format!(" {line}"));
    quote! { #(#[doc = #lines])* }
}

/// The columns of a line's leading spaces and tabs, a tab advancing to the
/// next multiple of four as in CommonMark, and the rest of the line.
fn indentation(line: &str) -> (usize, &str) {
    let mut columns = 0;
    for (index, c) in line.char_indices() {
        match c {
            ' ' => columns += 1,
            '\t' => columns += 4 - columns % 4,
            _ => return (columns, &line[index..]),
        }
    }
    (columns, "")
}

/// The character and length of a Markdown code fence that `text`, a line
/// indented by fewer than four columns, opens.
fn fence(text: &str) -> Option<(char, usize)> {
    let c = text.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let count = text.chars().take_while(|next| *next == c).count();
    // A backtick fence's info string has no backticks; with one, the line
    // is ordinary text with code spans.
    (count >= 3 && (c == '~' || !text[count..].contains('`'))).then_some((c, count))
}

/// Whether `line` closes a fence of `length` `c` characters.
fn closes_fence(line: &str, c: char, length: usize) -> bool {
    let (indent, text) = indentation(line);
    let count = text.chars().take_while(|next| *next == c).count();
    indent < 4 && count >= length && text[count..].trim().is_empty()
}

/// The column at which the content of a list item that `text` starts
/// begins, counted from its marker: the marker and the spaces after it, or
/// only one of them before an indented code block or nothing.
fn list_item(text: &str) -> Option<usize> {
    let digits = text.bytes().take_while(u8::is_ascii_digit).count();
    let marker = if text.starts_with(['-', '*', '+']) {
        1
    } else if (1..=9).contains(&digits) && text[digits..].starts_with(['.', ')']) {
        digits + 1
    } else {
        return None;
    };
    let (spaces, content) = indentation(&text[marker..]);
    match spaces {
        0 if content.is_empty() => Some(marker + 1),
        0 => None,
        1..=4 if !content.is_empty() => Some(marker + spaces),
        _ => Some(marker + 1),
    }
}

/// Whether a line starts a Markdown block of its own (a list item, heading,
/// thematic break or setext underline), which ends the text before it, so
/// that no code span continues across it.
fn starts_block(text: &str) -> bool {
    let text = text.trim_end();
    let hashes = text.bytes().take_while(|b| *b == b'#').count();
    let digits = text.bytes().take_while(u8::is_ascii_digit).count();
    let marker = if text.starts_with(['-', '*', '+']) {
        1
    } else if (1..=6).contains(&hashes) {
        hashes
    } else if (1..=9).contains(&digits) && text[digits..].starts_with(['.', ')']) {
        digits + 1
    } else {
        0
    };
    let rule = |c: char| {
        let marks = text.chars().filter(|next| *next == c).count();
        marks >= if c == '=' || c == '-' { 1 } else { 3 }
            && text
                .chars()
                .all(|next| next == c || next == ' ' || next == '\t')
    };
    (marker > 0 && (text.len() == marker || text[marker..].starts_with([' ', '\t'])))
        || ['=', '-', '*', '_'].into_iter().any(rule)
}

/// The length of the longest run of `c` in `text`.
fn longest_run(text: &str, c: char) -> usize {
    text.split(|next| next != c)
        .map(|run| run.chars().count())
        .max()
        .unwrap_or(0)
}

/// `text` as a Markdown code span.
fn code_span(text: &str) -> String {
    let ticks = "`".repeat(longest_run(text, '`') + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{ticks}{pad}{text}{pad}{ticks}")
}

/// Render QAPI's reStructuredText documentation as Markdown that rustdoc
/// shows as written, without doctests.
///
/// Literal blocks become `text` code blocks: indented lines where Markdown
/// would open an indented code block (at the start, after a blank line or
/// after another block) or that follow a paragraph ending in `::`, with the
/// blank lines between them. Explicit fences are kept and become `text`
/// blocks too, whatever their info string. Prose goes through [`prose`].
///
/// rustdoc removes the space after `///` only when every line of the item's
/// documentation starts with one, which a doc line that a macro adds can
/// prevent. So prose indented by three or more columns, and fences, lose
/// their indentation: the prose continues its paragraph, and nothing that
/// is not fenced can read as code either way. The later lines of a list
/// item's paragraph are indented to the item's content, where Markdown,
/// and Clippy's `doc_lazy_continuation`, expect them.
fn render_doc(doc: &str) -> Vec<String> {
    let lines: Vec<&str> = doc.lines().collect();
    let mut out = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();
    // The indentation of a paragraph that ends in `::`, which a more
    // indented literal block follows.
    let mut literal_after = None;
    let mut paragraph_indent = 0;
    // The output column of the content of the list item that the paragraph
    // starts, if it starts one.
    let mut item_content = None;
    let mut index = 0;
    while index < lines.len() {
        let (indent, text) = indentation(lines[index]);
        if text.is_empty() {
            out.extend(prose(&std::mem::take(&mut paragraph)));
            out.push(String::new());
            index += 1;
            continue;
        }
        if indent < 4
            && let Some((c, length)) = fence(text)
        {
            out.extend(prose(&std::mem::take(&mut paragraph)));
            literal_after = None;
            // Unindented, as Markdown unindents the content by the
            // fence's indentation.
            let fence = &text[..length];
            out.push(format!("{fence}text"));
            index += 1;
            while index < lines.len() && !closes_fence(lines[index], c, length) {
                let (columns, content) = indentation(lines[index]);
                out.push(format!(
                    "{}{content}",
                    " ".repeat(columns.saturating_sub(indent))
                ));
                index += 1;
            }
            out.push(fence.to_owned());
            index += 1;
            continue;
        }
        let threshold = match literal_after {
            Some(marker) if indent > marker => Some(4.min(marker + 1)),
            _ if paragraph.is_empty() && indent >= 4 => Some(4),
            _ => None,
        };
        if let Some(threshold) = threshold {
            out.extend(prose(&std::mem::take(&mut paragraph)));
            literal_after = None;
            let start = index;
            while index < lines.len() {
                let (indent, text) = indentation(lines[index]);
                if text.is_empty() {
                    // Blank lines belong to the block when it continues after them.
                    let next = lines[index..]
                        .iter()
                        .map(|line| indentation(line))
                        .find(|(_, text)| !text.is_empty());
                    if !next.is_some_and(|(indent, _)| indent >= threshold) {
                        break;
                    }
                } else if indent < threshold {
                    break;
                }
                index += 1;
            }
            literal_block(&lines[start..index], &mut out);
            continue;
        }
        if starts_block(text) || fence(text).is_some() {
            out.extend(prose(&std::mem::take(&mut paragraph)));
        }
        let kept = if indent >= 3 { 0 } else { indent };
        let text = if paragraph.is_empty() {
            paragraph_indent = indent;
            item_content = list_item(text).map(|width| kept + width);
            format!("{}{text}", " ".repeat(kept))
        } else {
            // A list item's lines continue at its content's column, which
            // Markdown would otherwise read as a lazy continuation.
            format!("{}{text}", " ".repeat(item_content.unwrap_or(kept)))
        };
        literal_after = (text.trim_end().ends_with("::") && !text.trim_start().starts_with(".."))
            .then_some(paragraph_indent);
        paragraph.push(text);
        index += 1;
    }
    out.extend(prose(&paragraph));
    out
}

/// Lines in a `text` code block, dedented by their common indentation,
/// fenced with more backticks than any run in them.
fn literal_block(lines: &[&str], out: &mut Vec<String>) {
    let lines: Vec<_> = lines.iter().map(|line| indentation(line)).collect();
    let common = lines
        .iter()
        .filter(|(_, text)| !text.is_empty())
        .map(|(indent, _)| *indent)
        .min()
        .unwrap_or(0);
    let longest = lines
        .iter()
        .map(|(_, text)| longest_run(text, '`'))
        .max()
        .unwrap_or(0);
    let fence = "`".repeat(longest.max(2) + 1);
    out.push(format!("{fence}text"));
    for (indent, text) in lines {
        out.push(if text.is_empty() {
            String::new()
        } else {
            format!("{}{text}", " ".repeat(indent - common))
        });
    }
    out.push(fence);
}

/// Whether `text` is an `http` or `https` URL.
fn is_url(text: &str) -> bool {
    ["https://", "http://"]
        .iter()
        .any(|scheme| text.len() > scheme.len() && text.starts_with(scheme))
}

/// The length of a Markdown link to a URL, `[label](https://...)`, at the
/// start of `text`. Other destinations would be rustdoc intra-doc links.
fn markdown_link(text: &str) -> Option<usize> {
    let label = text[1..].find(['[', ']', '<', '>', '`', '\\', '\n'])? + 1;
    if label == 1 || !text[label..].starts_with("](") {
        return None;
    }
    let destination = &text[label + 2..];
    let end = destination
        .find(|c: char| c.is_whitespace() || matches!(c, '(' | ')' | '<' | '>' | '`'))?;
    (destination[end..].starts_with(')') && is_url(&destination[..end]))
        .then_some(label + 2 + end + 1)
}

/// The length of a Markdown autolink, `<https://...>`, at the start of `text`.
fn autolink(text: &str) -> Option<usize> {
    let inner = &text[1..];
    let end = inner
        .find(|c: char| c.is_whitespace() || c.is_control() || matches!(c, '<' | '>' | '`'))?;
    (inner[end..].starts_with('>') && is_url(&inner[..end])).then_some(end + 2)
}

/// A bare URL at the start of `text`, without the punctuation that ends
/// the sentence around it.
fn bare_url(text: &str) -> Option<&str> {
    if !text.starts_with("https://") && !text.starts_with("http://") {
        return None;
    }
    let mut url = &text[..text.find(char::is_whitespace).unwrap_or(text.len())];
    loop {
        let mut trimmed =
            url.trim_end_matches(['.', ',', ':', ';', '!', '?', '\'', '"', '*', '_', '~']);
        for (open, close) in [('(', ')'), ('[', ']')] {
            if trimmed.ends_with(close)
                && trimmed.matches(open).count() < trimmed.matches(close).count()
            {
                trimmed = &trimmed[..trimmed.len() - 1];
            }
        }
        if trimmed == url {
            break;
        }
        url = trimmed;
    }
    is_url(url).then_some(url)
}

/// Whether `text`, after an `&`, would make it an HTML entity reference.
fn entity(text: &str) -> bool {
    let name = text.strip_prefix('#').map_or(text, |number| {
        number.strip_prefix(['x', 'X']).unwrap_or(number)
    });
    let end = name
        .find(|c: char| !c.is_ascii_alphanumeric())
        .unwrap_or(name.len());
    end > 0 && name[end..].starts_with(';')
}

/// Render a paragraph of QAPI prose as Markdown that rustdoc shows as
/// written: brackets, angle brackets, entity references and backticks
/// that open no code span are escaped, so that nothing becomes a link,
/// HTML or code by accident; code spans, also across lines, and backslash
/// escapes are kept; URLs become autolinks, or code where they contain
/// angle brackets or backticks, and existing autolinks and links to URLs
/// stay links. A line that would open a code fence is escaped.
fn prose(lines: &[String]) -> Vec<String> {
    if lines.is_empty() {
        return Vec::new();
    }
    let text = lines.join("\n");
    let mut out = String::with_capacity(text.len());
    let mut rest = text.as_str();
    let mut line_start = true;
    while let Some(c) = rest.chars().next() {
        let mut copy = 0;
        if c == '\\' && rest[1..].starts_with(|next: char| next.is_ascii_punctuation()) {
            copy = 2;
        } else if c == '`' {
            let run = rest.bytes().take_while(|byte| *byte == b'`').count();
            // A run of three at the start of a line would open a fence.
            match closing_ticks(&rest[run..], run).filter(|_| !(line_start && run >= 3)) {
                Some(end) => copy = 2 * run + end,
                None => {
                    out.push_str(&"\\`".repeat(run));
                    rest = &rest[run..];
                    line_start = false;
                    continue;
                }
            }
        } else if c == '[' {
            copy = markdown_link(rest).unwrap_or(0);
        } else if c == '<' {
            copy = autolink(rest).unwrap_or(0);
        } else if let Some(url) = bare_url(rest) {
            if url.contains(['<', '>', '`']) {
                out.push_str(&code_span(url));
            } else {
                out.push_str(&format!("<{url}>"));
            }
            rest = &rest[url.len()..];
            line_start = false;
            continue;
        }
        if copy > 0 {
            out.push_str(&rest[..copy]);
            line_start = rest[..copy].ends_with('\n');
            rest = &rest[copy..];
            continue;
        }
        match c {
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' if entity(&rest[1..]) => out.push_str("\\&"),
            '~' if line_start && rest.starts_with("~~~") => out.push_str("\\~"),
            _ => out.push(c),
        }
        line_start = c == '\n' || (line_start && c == ' ');
        rest = &rest[c.len_utf8()..];
    }
    out.split('\n').map(String::from).collect()
}

/// The offset in `text` of the next run of exactly `run` backticks.
fn closing_ticks(text: &str, run: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'`' {
            let count = bytes[index..]
                .iter()
                .take_while(|byte| **byte == b'`')
                .count();
            if count == run {
                return Some(index);
            }
            index += count;
        } else {
            index += 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qapi_ir::{EnumVariant, Metadata, StructField, process_union};
    use crate::{QapiUnion, with_parser_version};
    use std::collections::HashMap;

    fn rendered(doc: &str) -> Vec<String> {
        let tokens = doc_to_tokens(doc);
        let item: syn::ItemStruct = syn::parse2(quote! { #tokens struct Example; }).unwrap();
        item.attrs
            .iter()
            .map(|attr| {
                let syn::Meta::NameValue(value) = &attr.meta else {
                    panic!("doc attribute")
                };
                let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(value),
                    ..
                }) = &value.value
                else {
                    panic!("doc string")
                };
                value.value()
            })
            .collect()
    }

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(|line| format!(" {line}")).collect()
    }

    #[track_caller]
    fn renders(doc: &str, expected: &str) {
        assert_eq!(rendered(doc), lines(expected), "{doc:?}");
    }

    fn code(tokens: TokenStream) -> String {
        prettyplease::unparse(&syn::parse2(tokens).unwrap())
    }

    #[test]
    fn prose_stays_literal_text() {
        renders(
            r"Escaped \[4\] \<id\> \`text",
            r"Escaped \[4\] \<id\> \`text",
        );
        renders(
            "Bit[3:0] and child<subtype> &amp; R&D",
            r"Bit\[3:0\] and child&lt;subtype&gt; \&amp; R&D",
        );
        renders(
            "Use `Bit[4] <id>` or ``[literal]<id>``.",
            "Use `Bit[4] <id>` or ``[literal]<id>``.",
        );
        // A code span continues on the next line of its paragraph.
        renders("x ``foo\nbar[0]`` y [1]", "x ``foo\nbar[0]`` y \\[1\\]");
        // Backticks that close nothing are text, and so is what follows.
        renders("a ` b [c]", r"a \` b \[c\]");
        renders("a `` b ` [c]", r"a \`\` b \` \[c\]");
        // No code span crosses into a list item, which Markdown starts there.
        renders("a `b\n- c` [d]", "a \\`b\n- c\\` \\[d\\]");
    }

    #[test]
    fn links_stay_links_only_where_they_lead_to_urls() {
        renders(
            "[link](https://example.test/) and <https://example.test/>",
            "[link](https://example.test/) and <https://example.test/>",
        );
        // Each bracket pair is a label of its own.
        renders(
            "[a] b [c](https://d.test/)",
            r"\[a\] b [c](https://d.test/)",
        );
        // A link to anything else would be an intra-doc link.
        renders("[a](b) <c>", r"\[a\](b) &lt;c&gt;");
    }

    #[test]
    fn urls_become_autolinks_or_code() {
        renders(
            "https://example.test/path#anchor",
            "<https://example.test/path#anchor>",
        );
        renders(
            "See https://a.test/x, or (https://b.test/y).",
            "See <https://a.test/x>, or (<https://b.test/y>).",
        );
        renders(
            "https://a.test/<b> and https://c.test/`d`",
            "`https://a.test/<b>` and `` https://c.test/`d` ``",
        );
        renders("<https://a.test/ b>", "&lt;<https://a.test/> b&gt;");
    }

    #[test]
    fn indented_lines_open_text_blocks_where_markdown_opens_code() {
        // Within a paragraph they continue it, dedented.
        renders(
            "@member: If present,\n    the rest\n\tof the sentence.",
            "@member: If present,\nthe rest\nof the sentence.",
        );
        // A list item's lines continue at its content's column.
        renders(
            "Errors:\n    - If one [x],\n      Cause\n    - Otherwise",
            "Errors:\n- If one \\[x\\],\n  Cause\n- Otherwise",
        );
        renders(
            "- a\n  - b\n    c\nd\n\n10.  e\n f\n\n-\n g",
            "- a\n  - b\n    c\n    d\n\n10.  e\n     f\n\n-\n  g",
        );
        // After a blank line, and across blank lines inside them.
        renders(
            "Example:\n\n    -> { \"a\": [1] }\n\n    <- {}\n\nAfter.",
            "Example:\n\n```text\n-> { \"a\": [1] }\n\n<- {}\n```\n\nAfter.",
        );
        // A tab indents as far as four spaces.
        renders(
            "para\n\n  \tlet x: u8 = 1;\n",
            "para\n\n```text\nlet x: u8 = 1;\n```",
        );
        // After a paragraph ending in `::`, any deeper indentation.
        renders(
            "   ::\n\n     -> {}\n     <- {}\n\n   With text.",
            "::\n\n```text\n-> {}\n<- {}\n```\n\nWith text.",
        );
        renders(
            "Data::\n   raw [x]\nNext",
            "Data::\n```text\nraw [x]\n```\nNext",
        );
        // Directives end in `::` too, but their content is prose.
        // Two columns stay, for nested lists.
        renders(
            ".. note::\n\n   A note [x].\n\n- a\n  - b",
            ".. note::\n\nA note \\[x\\].\n\n- a\n  - b",
        );
        // Backticks inside a block cannot close its fence.
        renders("\n    ```\n    x", "\n````text\n```\nx\n````");
    }

    #[test]
    fn explicit_fences_become_text_blocks() {
        renders(
            "Text [4]\n```\n    <id> [4]\n```\nAfter <id>",
            "Text \\[4\\]\n```text\n    <id> [4]\n```\nAfter &lt;id&gt;",
        );
        renders("~~~~ rust\nfn x() {}\n~~~~", "~~~~text\nfn x() {}\n~~~~");
        // An unclosed fence is closed at the end.
        renders("```\nx", "```text\nx\n```");
        // An indented fence and its content lose the fence's indentation.
        renders("  ```\n  x\n   y\n  ```", "```text\nx\n y\n```");
        // Fence-like text inside a paragraph is escaped.
        renders("a\n    ```\n    ~~~ b", "a\n\\`\\`\\`\n\\~~~ b");
    }

    #[test]
    fn items_render_with_their_attributes_documentation_and_types() {
        let meta = |doc: Option<&str>, attributes: Vec<Attribute>| Metadata {
            doc: doc.map(String::from),
            attributes,
            ..Metadata::default()
        };
        let enumeration = Enum {
            name: "Choice".into(),
            meta: meta(
                Some("Branches"),
                vec![
                    Attribute::new("union_branch"),
                    Attribute::with_value("added", "1.0.0"),
                    Attribute::nested(
                        "unstable",
                        vec![
                            Attribute::with_value("added", "2.0.0"),
                            Attribute::with_value("removed", "3.0.0"),
                        ],
                    ),
                ],
            ),
            variants: vec![
                EnumVariant {
                    name: "one-two".into(),
                    meta: meta(
                        Some("First"),
                        vec![
                            Attribute::with_value("name", "one-two"),
                            Attribute::with_values(vec![
                                ("feature", Some("unstable")),
                                ("condition", Some("CONFIG_X")),
                            ]),
                        ],
                    ),
                    kind: EnumVariantKind::Tuple("Payload".into()),
                    array: false,
                },
                EnumVariant {
                    name: "many".into(),
                    meta: meta(None, Vec::new()),
                    kind: EnumVariantKind::Tuple("str".into()),
                    array: true,
                },
                EnumVariant {
                    name: "Self".into(),
                    meta: meta(None, vec![Attribute::with_value("name", "Self")]),
                    kind: EnumVariantKind::Unit,
                    array: false,
                },
            ],
        };
        assert_eq!(
            code(enumeration.generate()),
            r#"/// Branches
#[qapi(union_branch)]
#[qapi(added = "1.0.0")]
#[qapi(unstable(added = "2.0.0", removed = "3.0.0"))]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum Choice {
    /// First
    #[qapi(name = "one-two")]
    #[qapi(feature = "unstable", condition = "CONFIG_X")]
    OneTwo(Payload),
    Many(Vec<String>),
    #[qapi(name = "Self")]
    QapiSelf,
}
"#
        );

        let field = |name: &str, r#type: &str, optional, array, doc, attributes| StructField {
            name: name.into(),
            meta: meta(doc, attributes),
            r#type: r#type.into(),
            optional,
            array,
        };
        let structure = Struct {
            name: "query-thing".into(),
            meta: meta(
                Some("A command"),
                vec![
                    Attribute::new("command"),
                    Attribute::with_value("name", "query-thing"),
                    Attribute::with_value("returns", "Vec<Thing>"),
                ],
            ),
            fields: vec![
                field(
                    "type",
                    "Kind",
                    false,
                    false,
                    Some("The kind"),
                    vec![Attribute::new("discriminator")],
                ),
                field(
                    "next",
                    "query-thing",
                    true,
                    false,
                    None,
                    vec![Attribute::new("boxed")],
                ),
                field("values", "int", true, true, None, Vec::new()),
                field("self", "any", false, false, None, Vec::new()),
            ],
        };
        assert_eq!(
            code(structure.generate()),
            r#"/// A command
#[qapi(command)]
#[qapi(name = "query-thing")]
#[qapi(returns = "Vec<Thing>")]
pub struct QueryThing {
    /// The kind
    #[qapi(discriminator)]
    pub r#type: Kind,
    pub next: Option<Box<QueryThing>>,
    pub values: Option<Vec<i64>>,
    pub self_: serde_json::Value,
}
"#
        );
    }

    #[test]
    fn historical_simple_union_arrays_render_as_vectors() {
        let (_, input) = with_parser_version((2, 0, 0), || {
            QapiUnion::parse("{ 'union': 'Example', 'data': {'text': 'str', 'values': ['int']} }")
        })
        .unwrap();
        let (_, branches, _) = with_parser_version((2, 0, 0), || {
            process_union(input, &HashMap::new(), &HashMap::new())
        })
        .unwrap();
        assert!(code(branches.generate()).contains("Values(Vec<i64>)"));
    }
}
