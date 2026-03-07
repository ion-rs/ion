use ion::{Ion, ion};
use std::fs;
use std::path::Path;

fn read_ion(path: impl AsRef<Path>) -> ion::Ion {
    ion!(fs::read_to_string(path).unwrap())
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
    let expected = fs::read_to_string("tests/expected/test.ion").unwrap();

    assert_eq!(expected, ion.to_string());
}

#[test]
fn hotel_ion() {
    let ion = read_ion("tests/data/hotel.ion");
    let expected = fs::read_to_string("tests/expected/hotel.ion").unwrap();

    assert_eq!(expected, ion.to_string());
}

#[test]
fn broken_array_and_eof() {
    let ion_err = read_err_ion("tests/data/broken_array_and_eof.ion");

    let expected =
        "ParserErrors([ParserError { lo: 55, hi: 55, desc: \"Cannot finish an array\" }])";

    assert_eq!(expected, ion_err.to_string());
}

#[test]
fn broken_dictionary_and_eof() {
    let ion_err = read_err_ion("tests/data/broken_dictionary_and_eof.ion");

    let expected =
        "ParserErrors([ParserError { lo: 67, hi: 67, desc: \"Cannot finish a dictionary\" }])";

    assert_eq!(expected, ion_err.to_string());
}
