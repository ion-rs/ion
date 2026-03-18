#[macro_use]
mod ion;
mod parser;

pub use self::ion::*;
pub use self::parser::*;

#[cfg(feature = "dictionary-indexmap")]
pub type Dictionary = indexmap::IndexMap<String, Value>;
#[cfg(not(feature = "dictionary-indexmap"))]
pub type Dictionary = std::collections::BTreeMap<String, Value>;

#[cfg(feature = "dictionary-indexmap")]
pub type Sections = indexmap::IndexMap<String, Section>;
#[cfg(not(feature = "dictionary-indexmap"))]
pub type Sections = std::collections::BTreeMap<String, Section>;

pub type Row = Vec<Value>;
