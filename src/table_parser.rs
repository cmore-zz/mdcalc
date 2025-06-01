// table_parser.rs

use comrak::{nodes::{AstNode, NodeValue}, Arena};

use crate::comment_stripper::strip_comments_from_line;
use crate::md_comments::HtmlComment;

pub struct TableCell {
    pub raw: String,
    pub comments: Vec<HtmlComment>,
}

pub struct TableRow {
    pub cells: Vec<TableCell>,
}

pub struct MarkdownTable {
    pub rows: Vec<TableRow>,
}

pub struct TableParser;

impl TableParser {
    pub fn extract_tables_from_ast<'a>(root: &'a AstNode<'a>) -> Vec<MarkdownTable> {
        let mut tables = Vec::new();

        for node in root.descendants() {
            if let NodeValue::Paragraph = &node.data.borrow().value {
                if let Some(text) = Self::collect_literal_text(node) {
                    let lines: Vec<&str> = text.lines().filter(|l| l.starts_with('|')).collect();
                    if lines.len() >= 2 {
                        let table = Self::parse_table_lines(&lines);
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

    fn parse_table_lines(lines: &[&str]) -> MarkdownTable {
        let mut rows = Vec::new();
        for line in lines {
            if line.trim().is_empty() || line.contains("---") {
                continue;
            }

            let stripped = strip_comments_from_line(line);
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
                    .filter(|c| c.offset >= cell_start && c.offset < cell_end)
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

