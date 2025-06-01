// comment_stripper.rs

use crate::md_comments::{extract_html_comments, HtmlComment};

#[derive(Debug)]
pub struct CommentStrippedLine {
    pub original: String,
    pub stripped: String,
    pub comments: Vec<HtmlComment>,
}

/// Strips HTML comments from a line, replacing them with visible placeholders.
/// Returns the cleaned line and the extracted comments.
pub fn strip_comments_from_line(line: &str) -> CommentStrippedLine {
    let mut stripped = line.to_string();
    let comments = extract_html_comments(line);

    // Sort comments by descending offset so we can replace without shifting
    let mut sorted_comments = comments.clone();
    sorted_comments.sort_by_key(|c| -(c.offset as isize));

    for comment in &sorted_comments {
        let placeholder = format!("/*{}*/", comment.content);
        stripped.replace_range(
            comment.offset..comment.offset + comment.length,
            &placeholder,
        );
    }

    CommentStrippedLine {
        original: line.to_string(),
        stripped,
        comments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_and_replace() {
        let input = "A <!-- !A --> | B <!-- !=B2*C2 --> | C";
        let result = strip_comments_from_line(input);
        assert_eq!(result.comments.len(), 2);
        assert!(result.stripped.contains("/*!A*/"));
        assert!(result.stripped.contains("/*!==B2*C2*/"));
    }
}
