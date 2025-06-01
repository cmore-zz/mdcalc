
mod md_comments;

fn main() {
    let arena = comrak::Arena::new();
    let markdown = "Here is <!-- !A --> and <!-- !=B2*C2 --> inline.";
    let comments = md_comments::parse_markdown_for_comments(&arena, markdown);

    for located in comments {
        println!("{:?}", located);
    }
}
