
mod md_comments;
mod table_parser;
mod comment_stripper;

use comrak::{parse_document, Arena, ComrakOptions};
use comrak::nodes::{AstNode, NodeValue};
use table_parser::TableParser;

fn dump_ast<'a>(node: &'a AstNode<'a>, indent: usize) {
    let pad = "  ".repeat(indent);
    let val = &node.data.borrow().value;
    let desc = match val {
        NodeValue::Document => "Document",
        NodeValue::Paragraph => "Paragraph",
        NodeValue::Text(s) => &format!("Text({})", s),
        NodeValue::HtmlInline(s) => &format!("HtmlInline({})", s),
        NodeValue::HtmlBlock(s) => &format!("HtmlBlock({})", s.literal),
        NodeValue::Code(s) => &format!("Code({})", s.literal),
        NodeValue::SoftBreak => "SoftBreak",
        NodeValue::LineBreak => "LineBreak",
        _ => "Other",
    };
    println!("{}- {}", pad, desc);
    for child in node.children() {
        dump_ast(child, indent + 1);
    }
}


fn check_comments() {
    let arena = comrak::Arena::new();
    let markdown = "Here is <!-- !A --> and <!-- !=B2*C2 --> inline.";
    let comments = md_comments::parse_markdown_for_comments(&arena, markdown);

    for located in comments {
        println!("{:?}", located);
    }
}


fn main() {
    check_comments();

    let markdown = r#"
| Item <!-- !A -->     | Price <!-- !B --> | Quantity <!-- !C --> | Total <!-- !D -->         |
|----------------------|-------------------|-----------------------|---------------------------|
| Apples <!-- !2 -->   | 2                 | 3                     | 6 <!-- !=B2*C2 -->        |
| Bananas <!-- !3 -->  | 1                 | 5                     | 5 <!-- !=B3*C3 -->        |
| Cherries <!-- !4 --> | 4                 | 2                     | 8 <!-- !=B4*C4 -->        |
| Total <!-- !5 -->    |                   |                       | 19 <!-- !=D2+D3+D4 -->    |
"#;

    let arena = Arena::new();
    let options = ComrakOptions::default();
    let root = parse_document(&arena, markdown, &options);

    dump_ast(root, 2);

    let tables = TableParser::extract_tables_from_ast(root);
    println!("Found {} tables", tables.len());

    for (i, table) in tables.iter().enumerate() {
        println!("Table {}: {} rows", i + 1, table.rows.len());
        for row in &table.rows {
            for cell in &row.cells {
                print!("[{}] ", cell.raw);
            }
            println!();
        }
    }
}
