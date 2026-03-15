#[test]
#[cfg(not(feature = "dictionary-indexmap"))]
fn cli_tests_btree() {
    trycmd::TestCases::new().case("README.md");
}

#[test]
#[cfg(feature = "dictionary-indexmap")]
fn cli_tests_indexmap() {
    trycmd::TestCases::new().case("README.indexmap.md");
}
