use crate::toml::Pos;
use std::collections::HashMap;
use toml::Value;

pub mod toml;

#[derive(Debug)]
pub enum BlackDwarfError {
    UnknownKey {
        key: String,
        where_: Pos,
    },

    UnknownFileGroup {
        what: String,
        where_: Pos,
    },

    MissingKey {
        key: &'static str,
        where_: Pos,
    },

    IncorrectType {
        type_: &'static str,
        expected: &'static str,
        where_: Pos,
    },

    ParseError {
        why: String,
        where_: Pos,
    },
}

#[derive(Debug)]
pub struct FileGroup<'doc> {
    pub name: &'doc str,
    pub groups: Vec<&'doc str>,
    pub files: Vec<&'doc str>,
}

impl<'doc> FileGroup<'doc> {
    fn new(name: &'doc str) -> Self {
        FileGroup {
            name,
            groups: vec![],
            files: vec![],
        }
    }

    fn files(name: &'doc str, files: Vec<&'doc str>) -> Self {
        FileGroup {
            name,
            files,
            groups: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Target<'doc> {
    pub name: &'doc str,
    pub groups: Vec<&'doc str>,
    pub files: Vec<&'doc str>,
}

#[derive(Debug)]
pub struct BlackDwarf<'doc> {
    pub file_groups: HashMap<&'doc str, FileGroup<'doc>>,
    pub targets: HashMap<&'doc str, Target<'doc>>,
}

impl BlackDwarf<'_> {
    fn new() -> Self {
        BlackDwarf {
            file_groups: Default::default(),
            targets: Default::default(),
        }
    }

    fn has_file_group(&self, name: &str) -> bool {
        self.file_groups.contains_key(name)
    }
}

fn ensure_all_string(list: &[Value]) -> Result<(), BlackDwarfError> {
    for value in list {
        if !value.is_str() {
            return Err(BlackDwarfError::IncorrectType {
                type_: value.type_str(),
                expected: "string",
                where_: value.pos(),
            });
        }
    }

    Ok(())
}

impl<'doc, 'value: 'doc> TryFrom<&'value Value<'doc>> for BlackDwarf<'doc> {
    type Error = BlackDwarfError;
    fn try_from(value: &'value Value<'doc>) -> Result<Self, Self::Error> {
        let mut this = BlackDwarf::new();

        if let Some(file_groups) = value.get("file-groups") {
            if !file_groups.is_table() {
                return Err(BlackDwarfError::IncorrectType {
                    type_: file_groups.type_str(),
                    expected: "table",
                    where_: file_groups.pos(),
                });
            }

            for (name, contents) in file_groups.iter_kvs() {
                if let Some(files) = contents.as_list() {
                    // TODO check files exist
                    ensure_all_string(files)?;
                    this.file_groups.insert(
                        name,
                        FileGroup::files(
                            name,
                            files
                                .into_iter()
                                .map(Value::as_str)
                                .map(Option::unwrap)
                                .collect(),
                        ),
                    );
                } else if contents.is_table() {
                    // TODO warn unused kvs
                    let mut file_group = FileGroup::new(name);

                    if let Some(groups) = contents.get("groups").and_then(Value::as_list) {
                        ensure_all_string(groups)?;

                        for group in groups {
                            if !this.has_file_group(group.as_str().unwrap()) {
                                return Err(BlackDwarfError::UnknownFileGroup {
                                    what: group.as_str().unwrap().into(),
                                    where_: group.pos(),
                                });
                            }
                        }

                        file_group
                            .groups
                            .extend(groups.iter().map(Value::as_str).map(Option::unwrap));
                    }

                    if let Some(files) = contents.get("files").and_then(Value::as_list) {
                        ensure_all_string(files)?;
                        file_group
                            .files
                            .extend(files.iter().map(Value::as_str).map(Option::unwrap));
                    }

                    this.file_groups.insert(name, file_group);
                } else {
                    return Err(BlackDwarfError::IncorrectType {
                        type_: contents.type_str(),
                        expected: "table or array",
                        where_: contents.pos(),
                    });
                }
            }
        }

        Ok(this)
    }
}

#[test]
fn test_bd() {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bd_tests_dir = crate_dir.join("tests");
    let should_fail_dir = bd_tests_dir.join("should_fail");

    toml::for_each_toml_in_dir(&crate_dir, &bd_tests_dir, |name, contents| {
        println!("check bd {}", name);
        let toml = toml::parse(&contents).unwrap();
        let _bd = BlackDwarf::try_from(&toml).unwrap();
    });

    toml::for_each_toml_in_dir(&crate_dir, &should_fail_dir, |name, contents| {
        println!("check bd {}, should fail", name);
        let toml = toml::parse(&contents).unwrap();
        let _bd = BlackDwarf::try_from(&toml).unwrap_err();
    });
}
