// table_parser.rs

use comrak::{nodes::{AstNode, NodeValue}, Arena};

pub struct TableCell {
    pub raw: String,
    pub comments: Vec<String>,
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
                    let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| l.starts_with('|')).collect();
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
            let raw_cells: Vec<&str> = line.trim().trim_matches('|').split('|').collect();
            let cells = raw_cells
                .into_iter()
                .map(|cell| TableCell {
                    raw: cell.trim().to_string(),
                    comments: Vec::new(),
                })
                .collect();
            rows.push(TableRow { cells });
        }
        MarkdownTable { rows }
    }
}
