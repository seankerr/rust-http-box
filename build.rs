extern crate skeptic;

fn main() {
    skeptic::generate_doc_tests(&[
        "README.md",
        "examples/http_utilities.md",
        "examples/http1_head_parsing.md",
        "examples/http1_intro.md"
    ]);
}
