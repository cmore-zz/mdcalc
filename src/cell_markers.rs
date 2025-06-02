use crate::table_parser::{MarkdownTable, TableCell,TableCellPiece, TableParser};
use comrak::nodes::{Ast, AstNode,LineColumn,NodeValue, Sourcepos};
use crate::md_comments::{self, parse_markdown_for_comments, LocatedHtmlComment, CommentKind, HtmlComment};
use comrak::{parse_document, Arena, ComrakOptions};
use std::cell::RefCell;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerMode {
    DeleteAll,
    UpdateExisting,
    OnlyRowAndColumn,
    AllMarkers,
}




pub fn make_html_comment_node<'a>(arena: &'a Arena<AstNode<'a>>) -> &'a AstNode<'a> {
    let root = parse_document(arena, "<!-- test -->", &ComrakOptions::default());
    root.first_child().unwrap()
}

pub fn apply_marker_mode<'a>(
    table: &mut MarkdownTable<'a>,
    mode: MarkerMode,
    fallback_node: &'a AstNode<'a>,
) {
    let row_count = table.rows.len();
    if row_count == 0 {
        return;
    }

    for (row_idx, row) in table.rows.iter_mut().enumerate() {
        for (col_idx, cell) in row.cells.iter_mut().enumerate() {
            let is_header_row = row_idx == 0;
            let is_header_col = col_idx == 0;

            match mode {
                MarkerMode::DeleteAll => {
                    cell.pieces.retain(|p| {
                        !matches!(p, TableCellPiece::Comment(c) if c.comment.kind == CommentKind::Marker)
                    });
                }
                MarkerMode::UpdateExisting => {
                    for piece in &mut cell.pieces {
                        if let TableCellPiece::Comment(c) = piece {
                            if c.comment.kind == CommentKind::Marker {
                                c.comment.content = compute_marker(row_idx, col_idx);
                            }
                        }
                    }
                }
                MarkerMode::OnlyRowAndColumn => {
                    if is_header_row || is_header_col {
                        update_or_insert_marker(cell, row_idx, col_idx, fallback_node);
                    } else {
                        cell.pieces.retain(|p| {
                            !matches!(p, TableCellPiece::Comment(c) if c.comment.kind == CommentKind::Marker)
                        });
                    }
                }
                MarkerMode::AllMarkers => {
                    update_or_insert_marker(cell, row_idx, col_idx, fallback_node);
                }
            }
        }
    }
}



fn update_or_insert_marker<'a>(
    cell: &mut TableCell<'a>,
    row: usize,
    col: usize,
    fallback_node: &'a AstNode<'a>,
) {
    let marker = compute_marker(row, col);

    for piece in cell.pieces.iter_mut() {
        if let TableCellPiece::Comment(comment) = piece {
            if comment.comment.kind == CommentKind::Marker {
                comment.comment.content = marker;
                return;
            }
        }
    }

    // If no existing marker found, insert a new one
    cell.pieces.push(TableCellPiece::Comment(LocatedHtmlComment {
        node: fallback_node,
        comment: HtmlComment {
            content: marker,
            kind: CommentKind::Marker,
            offset: 0,
            length: 0,
        },
    }));
}

fn compute_marker(row: usize, col: usize) -> String {
    let col_char = (b'A' + (col as u8)) as char;
    let row_number = row + 1;
    format!("!{}{}", col_char, row_number)
}

pub fn print_table<'a>(table: &MarkdownTable<'a>) {
    for row in &table.rows {
        let rendered_cells: Vec<String> = row
            .cells
            .iter()
            .map(|cell| {
                cell.pieces
                    .iter()
                    .map(|piece| match piece {
                        TableCellPiece::Text(t) => t.clone(),
                        TableCellPiece::Comment(c) => format!("<!-- {} -->", c.comment.content.trim()),
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();

        println!("| {} |", rendered_cells.join(" | "));
    }
}

#[test]
fn test_only_row_and_column_markers() {
    let markdown = "\
| A | B | C |
|---|---|---|
| 1 | 2 | 3 |
| 4 | 5 | 6 |
";

    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &ComrakOptions::default());
    let comments = md_comments::parse_markdown_for_comments(&arena, markdown);
    let tables = TableParser::extract_tables_from_ast(root, Some(&comments), markdown);

    assert_eq!(tables.len(), 1);
    let mut table = tables[0].clone(); // Clone to allow mutation



    let fallback_node = make_html_comment_node(&arena);

    apply_marker_mode(&mut table, MarkerMode::OnlyRowAndColumn, fallback_node);

    print_table(&table);

    for (row_idx, row) in table.rows.iter().enumerate() {
        for (col_idx, cell) in row.cells.iter().enumerate() {
            let is_header_row = row_idx == 0;
            let is_header_col = col_idx == 0;
            let expected_marker = format!("!{}{}", (b'A' + col_idx as u8) as char, row_idx + 1);

            let cell_comments = cell.comments();

            if is_header_row || is_header_col {
                assert!(
                    cell_comments.iter().any(|c| c.comment.content == expected_marker),
                    "Expected marker '{}' in cell ({}, {})",
                    expected_marker, row_idx, col_idx
                );
            } else {
                assert!(
                    cell_comments.iter().all(|c| c.comment.kind != md_comments::CommentKind::Marker),
                    "Expected no markers in cell ({}, {})",
                    row_idx, col_idx
                );
            }
        }
    }
}
