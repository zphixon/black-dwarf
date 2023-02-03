use std::io::Read;

fn main() -> Result<(), ()> {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf).map_err(|_| ())?;
    let s = String::from_utf8(buf).map_err(|_| ())?;
    let toml = black_dwarf::toml::parse(&s).unwrap();
    println!("{}", toml.to_json());
    Ok(())
}

trait ToJson {
    fn to_json(self: Self) -> String;
}

impl ToJson for black_dwarf::toml::Value<'_> {
    fn to_json(self: Self) -> String {
        use black_dwarf::toml::Value::*;
        match self {
            Table { key_values, .. } => {
                let mut s: std::string::String = "{".into();
                for (k, v) in key_values {
                    s += &format!("\"{}\": {},", k, v.to_json());
                }
                s += "}";
                s
            }

            Array { values, .. } => {
                let mut s: std::string::String = "[".into();
                for v in values {
                    s += &v.to_json();
                    s += ",";
                }
                s += "]";
                s
            }

            String { value, .. } => format!("{{\"type\":\"string\",\"value\":\"{}\"}}", value),
            Integer { value, .. } => format!("{{\"type\":\"integer\",\"value\":\"{}\":\"}}", value),
            Float { value, .. } => format!("{{\"type\":\"float\",\"value\":\"{}\":\"}}", value),
            Boolean { value, .. } => format!("{{\"type\":\"bool\",\"value\":\"{}\":\"}}", value),

            Datetime { datetime, .. } => "{\"todo\":true}".into(),
        }
    }
}
