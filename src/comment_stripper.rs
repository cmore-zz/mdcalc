// comment_stripper.rs

use comrak::{Arena, parse_document, ComrakOptions};
use crate::md_comments::{extract_html_comments, parse_markdown_for_comments, HtmlComment, LocatedHtmlComment};

#[derive(Debug)]
pub struct CommentStrippedLine<'a> {
    pub original: String,
    pub stripped: String,
    pub comments: Vec<LocatedHtmlComment<'a>>,
}
/// Strips HTML comments from a line, replacing them with visible placeholders.
/// Returns the cleaned line and the extracted comments.
/// Strips HTML comments from a line, replacing them with visible placeholders.
/// Returns the cleaned line and the extracted comments.



pub fn strip_comments_from_line<'a>(
    line: &str,
    line_offset: usize,
    pre_parsed: Option<&[LocatedHtmlComment<'a>]>,
) -> CommentStrippedLine<'a> {
    let (relevant, mut stripped): (Vec<LocatedHtmlComment>, String) = if let Some(all_comments) = pre_parsed {
        println!("non_fallback pre_parsed: {:?}", pre_parsed);  // TEMP DEBUG
        let filtered = all_comments
            .iter()
            .filter_map(|lc| {
                let offset = lc.comment.offset;
                if offset >= line_offset && offset + lc.comment.length <= line_offset + line.len() {
                    let rel_offset = offset - line_offset;
                    let candidate = &line[rel_offset..rel_offset + lc.comment.length];
                    let expected = format!("<!--{}-->", lc.comment.content);
                    if candidate == expected {
                        let mut new_comment = lc.comment.clone();
                        new_comment.offset = rel_offset;
                        Some(LocatedHtmlComment {
                            node: lc.node,
                            comment: new_comment,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        (filtered, line.to_string())
    } else {
        let comments = extract_html_comments(line);
        println!("fallback extracted: {:?}", comments);  // TEMP DEBUG
        let located = comments
            .into_iter()
            .map(|comment| LocatedHtmlComment {
                node: panic!("No node context available in inline mode"),
                comment,
            })
            .collect();
        (located, line.to_string())
    };

    // Sort and replace in reverse
    let mut sorted = relevant.clone();
    sorted.sort_by_key(|c| -(c.comment.offset as isize));

    for c in &sorted {
        let sanitized_content = c.comment.content.replace('|', "¦");
        let placeholder = format!("/***{}**/", sanitized_content);
        stripped.replace_range(
            c.comment.offset..c.comment.offset + c.comment.length,
            &placeholder,
        );
    }

    CommentStrippedLine {
        original: line.to_string(),
        stripped,
        comments: relevant,
    }
}

pub fn strip_comments_from_line_with_comments<'a>(
    line: &str,
    all_comments: &'a [LocatedHtmlComment<'a>],
) -> CommentStrippedLine<'a> {
    let relevant = all_comments
        .iter()
        .filter(|c| c.comment.offset >= 0 && c.comment.offset < line.len()) // crude for now
        .cloned()
        .collect::<Vec<_>>();

    let mut stripped = line.to_string();
    for comment in &relevant {
        let replacement = "/***".to_string() + &" ".repeat(comment.comment.content.len()) + "**/";
        let start = comment.comment.offset;
        let end = start + comment.comment.length;
        stripped.replace_range(start..end, &replacement[..comment.comment.length]);
    }

    CommentStrippedLine {
        original: line.to_string(),
        stripped,
        comments: relevant,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_and_replace() {
        let input = "A <!-- !A --> | B <!-- =B2*C2 --> | C";
        let arena = Arena::new();
        let root = parse_document(&arena, input, &ComrakOptions::default());
        let comments = parse_markdown_for_comments(&arena, input);

        let result = strip_comments_from_line(input, 0, Some(&comments));
        println!("comments: {:?}, stripped: {}", result.comments, result.stripped);
        assert_eq!(result.comments.len(), 2);
        assert!(result.stripped.contains("/*** !A **/"));
        assert!(result.stripped.contains("/*** =B2*C2 **/"));
    }
}
