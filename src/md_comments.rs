// md_comments.rs

use comrak::{nodes::{AstNode, NodeValue}, parse_document, Arena, ComrakOptions};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentKind {
    Formula,    // starts with '=' after optional '!'
    Marker,     // starts with '!'
    Formatting, // starts with '$' or other
    Unknown,    // fallback
}

#[derive(Debug,Clone)]
pub struct HtmlComment {
    pub content: String,
    pub kind: CommentKind,
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug,Clone)]
pub struct LocatedHtmlComment<'a> {
    pub node: &'a AstNode<'a>,
    pub comment: HtmlComment,
}

pub fn extract_html_comments(html: &str) -> Vec<HtmlComment> {
    let mut comments = Vec::new();
    let mut start = 0;
    while let Some(begin) = html[start..].find("<!--") {
        if let Some(end) = html[start + begin..].find("-->") {
            let absolute_begin = start + begin;
            let absolute_end = absolute_begin + end + 3;
            let content_start = absolute_begin + 4;
            let content_end = absolute_begin + end;
            let full = &html[content_start..content_end];
            let trimmed = full.trim();

            let kind = if trimmed.starts_with("!=") || trimmed.starts_with('=') {
                CommentKind::Formula
            } else if trimmed.starts_with('!') &&
                    trimmed[1..].chars().all(|c| c.is_ascii_alphanumeric()) {
                CommentKind::Marker
            } else if trimmed.starts_with('%') {
                CommentKind::Formatting
            } else {
                CommentKind::Unknown
            };

            let comment = HtmlComment {
                content: full.to_string(),
                kind: kind,
                offset: absolute_begin,
                length: absolute_end - absolute_begin,
            };
            comments.push(comment);
            start = absolute_end;
        } else {
            break; // Malformed comment
        }
    }
    comments
}

pub fn parse_markdown_for_comments<'a>(
    arena: &'a Arena<AstNode<'a>>,
    markdown: &'a str,
) -> Vec<LocatedHtmlComment<'a>> {
    let options = ComrakOptions::default();
    let root = parse_document(arena, markdown, &options);
    let mut results = Vec::new();

    for node in root.descendants() {
        match &node.data.borrow().value {
            NodeValue::HtmlInline(html) => {
                results.extend(
                    extract_html_comments(html)
                        .into_iter()
                        .map(|comment| LocatedHtmlComment { node, comment }),
                );
            }
            NodeValue::HtmlBlock(block) => {
                results.extend(
                    extract_html_comments(&block.literal)
                        .into_iter()
                        .map(|comment| LocatedHtmlComment { node, comment }),
                );
            }
            _ => {}
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::Arena;

    #[test]
    fn test_extract_html_comments() {
        let input = "Here is <!-- !A --> and <!-- =B2*C2 --> inline.";
        let result = extract_html_comments(input);
        assert_eq!(result.len(), 2);
        assert_eq!(CommentKind::Marker, result[0].kind);
        assert_eq!(CommentKind::Formula, result[1].kind);
        println!("result[0].offset = {}", result[0].offset);

        assert_eq!(result[0].offset, 8);
        assert_eq!(result[1].offset, 24);
    }

    #[test]
    fn test_parse_markdown_for_comments() {
        let input = "Here is <!-- !A --> and <!-- =B2*C2 --> inline.";
        let arena = Arena::new();
        let result = parse_markdown_for_comments(&arena, input);
        assert_eq!(result.len(), 2);
    }
}


