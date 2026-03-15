#[macro_use]
mod ion;
mod parser;

pub use self::ion::*;
pub use self::parser::*;

#[cfg(feature = "dictionary-indexmap")]
pub type Dictionary = indexmap::IndexMap<String, Value>;
#[cfg(not(feature = "dictionary-indexmap"))]
pub type Dictionary = std::collections::BTreeMap<String, Value>;

pub type Row = Vec<Value>;
