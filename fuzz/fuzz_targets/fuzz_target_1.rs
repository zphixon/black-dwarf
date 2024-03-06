#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: String| {
    let _toml = black_dwarf::toml::parse(&data);
});
