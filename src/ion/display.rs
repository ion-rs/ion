use crate::{Ion, Section, Value};
use std::fmt;

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
                    write!(f, "\"")?;
                }

                let mut inside_quotes = f.alternate();

                for c in v.chars() {
                    if inside_quotes {
                        match c {
                            '\\' => write!(f, "\\\\")?,
                            '\n' => write!(f, "\\n")?,
                            '\"' => write!(f, "\\\"")?,
                            c => write!(f, "{c}")?,
                        }
                    } else if c == '"' {
                        write!(f, "\"")?;
                        inside_quotes = true;
                    } else {
                        write!(f, "{c}")?;
                    }
                }

                if f.alternate() {
                    write!(f, "\"")?;
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
