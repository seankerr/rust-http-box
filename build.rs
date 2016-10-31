extern crate skeptic;

fn main() {
    skeptic::generate_doc_tests(&["examples/detect_request_response.md"]);
    skeptic::generate_doc_tests(&["examples/headers.md"]);
    skeptic::generate_doc_tests(&["examples/query.md"]);
}
