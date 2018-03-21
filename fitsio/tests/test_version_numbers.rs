// Version number tests
//
// These tests will make sure the version numbers in the readme and main crate documentation stay
// in sync with the crate version number

#[macro_use]
extern crate version_sync;

#[test]
fn test_readme_deps() {
    assert_markdown_deps_updated!("../README.md");
}

#[test]
fn test_html_root_url() {
    assert_html_root_url_updated!("src/lib.rs");
}
