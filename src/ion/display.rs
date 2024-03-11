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
                    f.write_char('"')?;
                    for c in v.chars() {
                        match c {
                            '\\' => f.write_str("\\\\")?,
                            '\n' => f.write_str("\\n")?,
                            '\"' => f.write_str("\\\"")?,
                            _ => f.write_char(c)?,
                        }
                    }
                    f.write_char('"')?;
                } else {
                    let mut escaping = false;
                    for c in v.chars() {
                        match (escaping, c) {
                            (false, '\\') => {
                                escaping = true;
                                f.write_char('\\')?;
                                continue;
                            }
                            (false, '\n') => f.write_str("\\n")?,
                            (false, '\t') => f.write_str("\\t")?,
                            (false, '|') => f.write_str("\\|")?,

                            (true, '\\') => f.write_char('\\')?,
                            (true, 'n') => f.write_str("\\n")?,
                            (true, 't') => f.write_str("\\t")?,
                            (true, '|') => f.write_str("\\|")?,

                            (_, c) => f.write_char(c)?,
                        }
                        escaping = false;
                    }
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

                    write!(f, "{i:#}")?;
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

                    write!(f, "{v:#}")?;
                }

                f.write_str(" }")
            }
        }
    }
}
