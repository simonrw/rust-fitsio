// Version number tests
//
// These tests will make sure the version numbers in the readme and main crate documentation stay
// in sync with the crate version number

#[test]
fn test_readme_deps() {
    version_sync::assert_markdown_deps_updated!("../README.md");
}

#[test]
fn test_html_root_url() {
    version_sync::assert_html_root_url_updated!("src/lib.rs");
}
