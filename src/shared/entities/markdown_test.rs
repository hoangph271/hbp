use crate::shared::entities::markdown::extract_markdown_header_content;

#[test]
fn skip_if_no_metadata_comment() {
    assert_eq!(extract_markdown_header_content(""), None);
}
