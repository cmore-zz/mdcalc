use crate::table_parser::{MarkdownTable, TableCell};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerMode {
    DeleteAll,
    UpdateExisting,
    OnlyRowAndColumn,
    AllMarkers,
}

pub fn apply_marker_mode(table: &mut MarkdownTable, mode: MarkerMode) {
    let row_count = table.rows.len();
    if row_count == 0 {
        return;
    }
    let col_count = table.rows[0].cells.len();

    for (row_idx, row) in table.rows.iter_mut().enumerate() {
        for (col_idx, cell) in row.cells.iter_mut().enumerate() {
            let is_header_row = row_idx == 0;
            let is_header_col = col_idx == 0;

            match mode {
                MarkerMode::DeleteAll => {
                    cell.comments.retain(|c| c.comment.kind != crate::md_comments::CommentKind::Marker);
                }
                MarkerMode::UpdateExisting => {
                    for comment in &mut cell.comments {
                        if comment.comment.kind == crate::md_comments::CommentKind::Marker {
                            comment.comment.content = compute_marker(row_idx, col_idx);
                        }
                    }
                }
                MarkerMode::OnlyRowAndColumn => {
                    if is_header_row || is_header_col {
                        update_or_insert_marker(cell, row_idx, col_idx);
                    } else {
                        cell.comments.retain(|c| c.comment.kind != crate::md_comments::CommentKind::Marker);
                    }
                }
                MarkerMode::AllMarkers => {
                    update_or_insert_marker(cell, row_idx, col_idx);
                }
            }
        }
    }
}

fn update_or_insert_marker(cell: &mut TableCell, row: usize, col: usize) {
    let marker = compute_marker(row, col);
    if let Some(existing) = cell
        .comments
        .iter_mut()
        .find(|c| c.comment.kind == crate::md_comments::CommentKind::Marker)
    {
        existing.comment.content = marker;
    } else {
        cell.comments.push(crate::md_comments::LocatedHtmlComment {
            node: todo!("Insert real AstNode reference here"),
            comment: crate::md_comments::HtmlComment {
                content: marker,
                kind: crate::md_comments::CommentKind::Marker,
                offset: 0,
                length: 0,
            },
        });
    }
}

fn compute_marker(row: usize, col: usize) -> String {
    let col_char = (b'A' + (col as u8)) as char;
    let row_number = row + 1;
    format!("!{}{}", col_char, row_number)
}
