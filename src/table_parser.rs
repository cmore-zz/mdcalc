// table_parser.rs

use comrak::{nodes::{AstNode, NodeValue}, Arena};

use crate::comment_stripper::{strip_comments_from_line, strip_comments_from_line_with_comments, CommentStrippedLine};
use crate::md_comments::{HtmlComment, LocatedHtmlComment};

#[derive(Debug, Clone)]
pub enum TableCellPiece<'a> {
    Text(String),
    Comment(LocatedHtmlComment<'a>),
}

#[derive(Debug, Clone)]
pub struct TableCell<'a> {
    pub pieces: Vec<TableCellPiece<'a>>,
}

#[derive(Debug, Clone)]
pub struct TableRow<'a> {
    pub cells: Vec<TableCell<'a>>,
}

#[derive(Debug, Clone)]
pub struct MarkdownTable<'a> {
    pub rows: Vec<TableRow<'a>>,
    pub start_offset: usize,
    pub end_offset: usize,
}


impl<'a> TableCell<'a> {

    pub fn raw_text(&self) -> String {
        self.pieces.iter().filter_map(|p| {
            if let TableCellPiece::Text(s) = p {
                Some(s.as_str())
            } else {
                None
            }
        }).collect::<Vec<&str>>().join("")
    }

    pub fn comments(&self) -> Vec<&LocatedHtmlComment<'a>> {
        self.pieces.iter().filter_map(|p| {
            if let TableCellPiece::Comment(c) = p {
                Some(c)
            } else {
                None
            }
        }).collect()
    }

    pub fn text_content(&self) -> String {
        self.pieces.iter().filter_map(|p| {
            if let TableCellPiece::Text(t) = p {
                Some(t.as_str())
            } else {
                None
            }
        }).collect::<Vec<_>>().join("")
    }
}

pub struct TableParser;

impl TableParser {
    pub fn extract_tables_from_ast<'a>(
        root: &'a AstNode<'a>,
        comments: Option<&'a [LocatedHtmlComment<'a>]>,
        markdown: &'a str,
    ) -> Vec<MarkdownTable<'a>> {
        let mut tables = Vec::new();
        let lines: Vec<&str> = markdown.lines().collect();
        let line_offsets: Vec<usize> = lines
            .iter()
            .scan(0, |offset, line| {
                let current = *offset;
                *offset += line.len() + 1; // +1 for newline
                Some(current)
            })
            .collect();

        for node in root.descendants() {
            if let NodeValue::Paragraph = &node.data.borrow().value {
                if let Some(text) = Self::collect_literal_text(node) {
                    let start_line = node.data.borrow().sourcepos.start.line;
                    let end_line = node.data.borrow().sourcepos.end.line;
                    if let (Some(start_offset), Some(end_offset)) = (
                    line_offsets.get(start_line - 1),
                    line_offsets.get(end_line - 1).map(|o| o + lines[end_line - 1].len()),
                    ) {
                        let table_lines: Vec<&str> = text.lines().filter(|l| l.starts_with('|')).collect();
                        if table_lines.len() >= 2 {
                            let mut table = Self::parse_table_lines(&table_lines, comments);
                            table.start_offset = *start_offset;
                            table.end_offset = end_offset;
                            tables.push(table);
                        }
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

            let raw_cells: Vec<&str> = stripped
                .stripped
                .trim()
                .trim_matches('|')
                .split('|')
                .collect();

            // Track starting positions of each cell
            let mut cell_starts = Vec::new();
            let mut index = 0;
            for segment in &raw_cells {
                cell_starts.push(index);
                index += segment.len() + 1; // +1 for the '|' delimiter
            }

            if cell_starts.len() != raw_cells.len() {
                eprintln!(
                    "Warning: Mismatched cell starts vs raw cells. Skipping row: {}",
                    line
                );
                continue;
            }

            let mut cells = Vec::new();
            for (i, raw) in raw_cells.iter().enumerate() {
                let cell_start = cell_starts[i];
                let cell_end = cell_start + raw.len();
                let mut pieces = Vec::new();
                let mut last_offset = 0;

                for c in stripped
                    .comments
                    .iter()
                    .filter(|c| c.comment.offset >= cell_start && c.comment.offset < cell_end)
            {
                if c.comment.offset > last_offset {
                    let text_slice = &raw[last_offset..c.comment.offset - cell_start];
                    if !text_slice.trim().is_empty() {
                        pieces.push(TableCellPiece::Text(text_slice.to_string()));
                    }
                }
                pieces.push(TableCellPiece::Comment(c.clone()));
                last_offset = c.comment.offset - cell_start + c.comment.length;
            }

                if last_offset < raw.len() {
                    let remaining = &raw[last_offset..];
                    if !remaining.trim().is_empty() {
                        pieces.push(TableCellPiece::Text(remaining.to_string()));
                    }
                }

                cells.push(TableCell { pieces });
            }

            rows.push(TableRow { cells });
        }

        MarkdownTable {
            rows,
            start_offset: 0,
            end_offset: 0,
        }
    }    
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::md_comments::{parse_markdown_for_comments, CommentKind};
    use comrak::{parse_document, Arena, ComrakOptions};

    #[test]
    fn test_basic_table_with_comments() {
        let markdown = "\
| A <!-- !A --> | B <!-- =B2*C2 --> | C |
|--------------|------------------|---|
| 1            | 2                | 3 |
";

        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &ComrakOptions::default());
        let comments = parse_markdown_for_comments(&arena, markdown);
        let tables = TableParser::extract_tables_from_ast(root, Some(&comments), markdown);

        assert_eq!(tables.len(), 1);
        let table = &tables[0];
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0].cells.len(), 3);

        let cell = &table.rows[0].cells[0];

        let comments = cell.comments();
        let marker_comments: Vec<_> = comments
            .iter()
            .filter(|c| c.comment.kind == CommentKind::Marker)
            .collect();


        assert_eq!(marker_comments.len(), 1);
        assert!(marker_comments[0].comment.content.contains("!A"));
    }
}





