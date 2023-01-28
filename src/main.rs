use black_dwarf::{toml, BlackDwarf};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    let filename = if let Some(filename) = args.get(1) {
        filename.as_str()
    } else {
        "BD.toml"
    };

    let file = std::fs::read_to_string(filename).unwrap();
    let tokens = toml::scan(&file).unwrap();
    println!("{:#?}", tokens);

    let toml = toml::parse(&file).unwrap();
    println!("{:#?}", toml);

    let bd = BlackDwarf::try_from(&toml).unwrap();
    println!("bd={:?}", bd);
}
