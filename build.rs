extern crate skeptic;

fn main() {
    skeptic::generate_doc_tests(&["examples/detect_request_response.md",
                                  "examples/headers.md",
                                  "examples/query.md"]);
}
