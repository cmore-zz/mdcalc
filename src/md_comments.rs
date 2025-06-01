// md_comments.rs

use comrak::{nodes::{AstNode, NodeValue}, parse_document, Arena, ComrakOptions};

#[derive(Debug)]
pub struct HtmlComment {
    pub content: String,
    pub is_formula: bool,
    pub is_marker: bool,
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug)]
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
            let full = &html[content_start..content_end].trim();
            let comment = HtmlComment {
                content: full.to_string(),
                is_formula: full.starts_with('='),
                is_marker: full.len() == 1 && full.chars().all(|c| c.is_ascii_alphanumeric()),
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
        let input = "Here is <!-- !A --> and <!-- !=B2*C2 --> inline.";
        let result = extract_html_comments(input);
        assert_eq!(result.len(), 2);
        assert!(result[0].is_marker);
        assert!(result[1].is_formula);
        assert_eq!(result[0].offset, 9);
        assert_eq!(result[1].offset, 26);
    }

    #[test]
    fn test_parse_markdown_for_comments() {
        let input = "Here is <!-- !A --> and <!-- !=B2*C2 --> inline.";
        let arena = Arena::new();
        let result = parse_markdown_for_comments(&arena, input);
        assert_eq!(result.len(), 2);
    }
}


