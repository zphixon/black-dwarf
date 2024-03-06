use std::io::Read;

fn main() -> Result<(), ()> {
    let mut buf = Vec::new();
    let s;

    if let Some(filename) = std::env::args().nth(1) {
        s = std::fs::read_to_string(filename).map_err(|_| ())?;
    } else {
        std::io::stdin().read_to_end(&mut buf).map_err(|_| ())?;
        s = String::from_utf8(buf).map_err(|_| ())?;
    }

    match black_dwarf::toml::parse(&s) {
        Ok(toml) => {
            println!("{}", toml.to_json());
            Ok(())
        }
        Err(err) => {
            println!("{:?}", err);
            if std::env::args().any(|arg| arg == "--show-tokens-if-parse-failed") {
                println!("{:#?}", black_dwarf::toml::scan(&s));
            }
            Err(())
        }
    }
}

trait ToJson {
    fn to_json(self: Self) -> String;
}

impl ToJson for black_dwarf::toml::Value<'_> {
    fn to_json(self: Self) -> String {
        use black_dwarf::toml::{Datetime, Value};
        match self {
            Value::Table { key_values, .. } => {
                let mut s: String = "{".into();
                let len = key_values.len();
                for (i, (k, v)) in key_values.into_iter().enumerate() {
                    s += &format!("\"{}\": {}", k, v.to_json());
                    if i + 1 != len {
                        s += ",";
                    }
                }
                s += "}";
                s
            }

            Value::Array { values, .. } => {
                let mut s: String = "[".into();
                let len = values.len();
                for (i, v) in values.into_iter().enumerate() {
                    s += &v.to_json();
                    if i + 1 != len {
                        s += ",";
                    }
                }
                s += "]";
                s
            }

            Value::String { value, .. } => {
                format!(
                    "{{\"type\":\n\"string\",\"value\":\"{}\"}}",
                    value.replace("\\", "\\\\").replace("\n", "\\n").replace("\"", "\\\""),
                )
            }
            Value::Integer { value, .. } => {
                format!("{{\"type\":\n\"integer\",\"value\":\"{}\"}}", value)
            }
            Value::Float { value, .. } => {
                if !value.is_nan() {
                    format!("{{\"type\":\n\"float\",\"value\":\"{}\"}}", value)
                } else {
                    format!("{{\"type\":\n\"float\",\"value\":\"nan\"}}")
                }
            }
            Value::Boolean { value, .. } => {
                format!("{{\"type\":\n\"bool\",\"value\":\"{}\"}}", value)
            }

            Value::Datetime { datetime, .. } => {
                let value = datetime.to_string();
                match datetime {
                    Datetime {
                        date: Some(_),
                        time: Some(_),
                        offset: Some(_),
                    } => format!("{{\"type\":\n\"datetime\",\"value\":\"{}\"}}", value),

                    Datetime {
                        date: Some(_),
                        time: Some(_),
                        offset: None,
                    } => format!("{{\"type\":\n\"datetime-local\",\"value\":\"{}\"}}", value),

                    Datetime {
                        date: Some(_),
                        time: None,
                        offset: None,
                    } => format!("{{\"type\":\n\"date-local\",\"value\":\"{}\"}}", value),

                    Datetime {
                        date: None,
                        time: Some(_),
                        offset: None,
                    } => format!("{{\"type\":\n\"time-local\",\"value\":\"{}\"}}", value),

                    _ => unreachable!(),
                }
            }
        }
    }
}
