use indoc::indoc;
use ion::{Ion, ion};
use std::fs;
use std::path::Path;

#[cfg(feature = "dictionary-indexmap")]
const TEST_ION_EXPECTED_PATH: &str = "tests/expected/test.indexmap.ion";
#[cfg(not(feature = "dictionary-indexmap"))]
const TEST_ION_EXPECTED_PATH: &str = "tests/expected/test.ion";

#[cfg(feature = "dictionary-indexmap")]
const HOTEL_ION_EXPECTED_PATH: &str = "tests/expected/hotel.indexmap.ion";
#[cfg(not(feature = "dictionary-indexmap"))]
const HOTEL_ION_EXPECTED_PATH: &str = "tests/expected/hotel.ion";

fn read_ion(path: impl AsRef<Path>) -> ion::Ion {
    ion!(fs::read_to_string(path).unwrap())
}

fn read_expected_ion(path: impl AsRef<Path>) -> String {
    let mut expected = fs::read_to_string(path).unwrap();
    if !expected.ends_with("\n\n") {
        expected.push('\n');
    }
    expected
}

fn read_err_ion(path: impl AsRef<Path>) -> ion::IonError {
    fs::read_to_string(path)
        .unwrap()
        .parse::<ion::Ion>()
        .unwrap_err()
}

#[test]
fn test_ion() {
    let ion = read_ion("tests/data/test.ion");
    let expected = read_expected_ion(TEST_ION_EXPECTED_PATH);

    assert_eq!(expected, ion.to_string());
}

#[test]
fn hotel_ion() {
    let ion = read_ion("tests/data/hotel.ion");
    let expected = read_expected_ion(HOTEL_ION_EXPECTED_PATH);

    assert_eq!(expected, ion.to_string());
}

#[test]
fn broken_array_and_eof() {
    let ion_err = read_err_ion("tests/data/broken_array_and_eof.ion");

    let actual = ion_err.to_string();
    let expected = indoc! {r#"
        Cannot finish an array at line 3, column 17 (found end of input)
        markets = ["abc"
                        ^
    "#}
    .trim_end();
    assert_eq!(expected, actual);
}

#[test]
fn broken_dictionary_and_eof() {
    let ion_err = read_err_ion("tests/data/broken_dictionary_and_eof.ion");

    let actual = ion_err.to_string();
    let expected = indoc! {r#"
        Cannot finish a dictionary at line 3, column 24 (found end of input)
        markets = { foo = "bar"
                               ^
    "#}
    .trim_end();
    assert_eq!(expected, actual);
}
