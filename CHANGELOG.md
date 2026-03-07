# Changelog

## 0.9.1

- Fixed a couple of formatting edge-cases

## 0.9.0

- Added license
- Cleaned up API, removed `Writer`
- Renamed repository from `ion_rs` to `ion`

## 0.8.9

- Support escape `\` character when reading cells and strings

## 0.8.6

- Optimize parser a bit

## 0.8.5

- Remove unused `slice_pattern` feature
- Remove deprecated `try!` macro
- Use inclusive range syntax

## 0.8.1

- Add filtering of sections when loading ion

## 0.8

- Drop unused / unfinished features
- RustcDeserialize support
- Validator (which wasn't working anyway)

## 0.7.3

- Fix `Display` of `Value::String` withing arrays to be enclosed in `"`

## 0.7.1

- Fix slice pattern for `rustc 1.12.0-nightly (2ad5ed07f 2016-07-08)`

