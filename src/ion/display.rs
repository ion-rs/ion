use crate::{Ion, Section, Value};
use std::fmt::{self, Write};

impl fmt::Display for Ion {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (name, section) in &self.sections {
            f.write_fmt(format_args!("[{name}]\n"))?;
            section.fmt(f)?;
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (k, v) in &self.dictionary {
            f.write_fmt(format_args!("{k} = {v:#}\n"))?;
        }

        for row in &self.rows {
            for cell in row {
                fmt::Display::fmt(&format!("| {cell} "), f)?;
            }
            f.write_str("|\n")?;
        }

        Ok(())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Value::String(v) => {
                if f.alternate() {
                    f.write_str("\"")?;
                }

                for c in v.chars() {
                    match c {
                        '\\' => f.write_str(if f.alternate() { "\\\\" } else { "\\" })?,
                        '\n' => f.write_str("\\n")?,
                        '\"' => f.write_str(if f.alternate() { "\\\"" } else { "\"" })?,
                        _ => f.write_char(c)?,
                    }
                }

                if f.alternate() {
                    f.write_str("\"")?;
                }

                Ok(())
            }

            Value::Integer(v) => v.fmt(f),
            Value::Float(v) => v.fmt(f),
            Value::Boolean(v) => v.fmt(f),

            Value::Array(v) => {
                f.write_str("[ ")?;

                let mut first = true;

                for i in v {
                    if first {
                        first = false;
                    } else {
                        f.write_str(", ")?
                    }

                    i.fmt(f)?;
                }

                f.write_str(" ]")
            }

            Value::Dictionary(d) => {
                f.write_str("{ ")?;

                let mut first = true;

                for (k, v) in d {
                    if first {
                        first = false;
                    } else {
                        f.write_str(", ")?
                    }

                    k.fmt(f)?;
                    f.write_str(" = ")?;

                    v.fmt(f)?;
                }

                f.write_str(" }")
            }
        }
    }
}
