// table_parser.rs

use comrak::{nodes::{AstNode, NodeValue}, Arena};

use crate::comment_stripper::{strip_comments_from_line, strip_comments_from_line_with_comments, CommentStrippedLine};
use crate::md_comments::{HtmlComment, LocatedHtmlComment};

pub struct TableCell<'a> {
    pub raw: String,
    pub comments: Vec<LocatedHtmlComment<'a>>,
}

pub struct TableRow<'a> {
    pub cells: Vec<TableCell<'a>>,
}

pub struct MarkdownTable<'a> {
    pub rows: Vec<TableRow<'a>>,
}

pub struct TableParser;

impl TableParser {
    pub fn extract_tables_from_ast<'a>(
        root: &'a AstNode<'a>,
        comments: Option<&'a [LocatedHtmlComment<'a>]>,
    ) -> Vec<MarkdownTable<'a>> {
        let mut tables = Vec::new();

        for node in root.descendants() {
            if let NodeValue::Paragraph = &node.data.borrow().value {
                if let Some(text) = Self::collect_literal_text(node) {
                    let lines: Vec<&str> = text.lines().filter(|l| l.starts_with('|')).collect();
                    if lines.len() >= 2 {
                        let table = Self::parse_table_lines(&lines, comments);
                        tables.push(table);
                    }
                }
            }
        }

        tables
    }

    fn collect_literal_text<'a>(node: &'a AstNode<'a>) -> Option<String> {
        let mut text = String::new();
        for child in node.children() {
            match &child.data.borrow().value {
                NodeValue::Text(s) | NodeValue::HtmlInline(s) => {
                    text.push_str(s);
                }
                NodeValue::Code(code_node) => {
                    text.push_str(&code_node.literal);
                }
                NodeValue::SoftBreak | NodeValue::LineBreak => {
                    text.push('\n');
                }
                _ => {}
            }
        }
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    fn parse_table_lines<'a>(
        lines: &[&str],
        comments: Option<&'a [LocatedHtmlComment<'a>]>,
    ) -> MarkdownTable<'a> {
        let mut rows = Vec::new();
        for line in lines {
            if line.trim().is_empty() || line.contains("---") {
                continue;
            }

            let stripped: CommentStrippedLine<'a> = match comments {
                Some(all_comments) => strip_comments_from_line_with_comments(line, all_comments),
                None => strip_comments_from_line(line, 0, None),
            };

            let mut cell_starts = Vec::new();
            let mut index = 0;
            for segment in stripped.stripped.trim().trim_matches('|').split('|') {
                cell_starts.push(index);
                index += segment.len() + 1; // +1 for the '|' delimiter
            }

            let raw_cells: Vec<&str> = stripped.stripped.split('|').collect();
            let mut cells = Vec::new();
            for (i, raw) in raw_cells.iter().enumerate() {
                let cell_start = cell_starts[i];
                let cell_end = cell_start + raw.len();
                let comments = stripped
                    .comments
                    .iter()
                    .filter(|c| c.comment.offset >= cell_start && c.comment.offset < cell_end)
                    .cloned()
                    .collect();
                cells.push(TableCell {
                    raw: raw.to_string(),
                    comments,
                });
            }
            rows.push(TableRow { cells });
        }
        MarkdownTable { rows }
    }
} 





