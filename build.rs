extern crate skeptic;

fn main() {
    skeptic::generate_doc_tests(&[
        "README.md",
        "examples/http_decoding_url_encoded_data.md",
        "examples/http_parsing_header_fields.md",
        "examples/http_parsing_query_strings.md",
        "examples/http1_head_parsing.md",
        "examples/http1_intro.md"
    ]);
}
