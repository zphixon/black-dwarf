use black_dwarf::{toml, BlackDwarf};

fn main() {
    let file = std::fs::read_to_string("BD.toml").unwrap();
    let toml = toml::parse(&file).unwrap();
    println!("{:#?}", toml);

    let bd = BlackDwarf::try_from(&toml).unwrap();
    println!("bd={:?}", bd);
}
