use crate::{error::Error, UnusedKeys};
use indexmap::IndexMap;
use serde::{de::Visitor, Deserializer};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    str::FromStr,
};

#[derive(macros::UnusedKeys, serde::Deserialize, Debug)]
pub struct Project {
    pub project: ProjectMeta,
    pub target: IndexMap<String, Target>,

    #[serde(flatten)]
    #[unused]
    pub rest: HashMap<String, toml::Value>,
}

#[derive(macros::UnusedKeys, serde::Deserialize, Debug)]
pub struct ProjectMeta {
    pub name: String,
    pub version: String,

    #[serde(flatten)]
    #[unused]
    pub rest: HashMap<String, toml::Value>,
}

#[derive(Debug)]
pub enum TargetType {
    Static,
    Dynamic,
}

impl FromStr for TargetType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "static" => Ok(TargetType::Static),
            "dynamic" => Ok(TargetType::Dynamic),
            _ => Err(format!("Unknown target type {:?}", s)),
        }
    }
}

impl UnusedKeys for TargetType {
    fn unused_keys(&self) -> Vec<String> {
        vec![]
    }
}

macro_rules! bruh {
    ($name:ident, $tp:ty) => {
        fn $name<'de, D>(de: D) -> Result<Vec<$tp>, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct Visit;
            impl<'v> Visitor<'v> for Visit {
                type Value = Vec<$tp>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "string or list of string")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'v>,
                {
                    let mut value = Vec::new();
                    while let Some(next) = seq.next_element::<String>()? {
                        use serde::de::Error;
                        value.push(<$tp>::from_str(&next).map_err(|err| A::Error::custom(err))?);
                    }
                    Ok(value)
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    let target_type = <$tp>::from_str(v).map_err(|err| E::custom(err))?;
                    Ok(vec![target_type])
                }

                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    self.visit_str(&v)
                }

                fn visit_borrowed_str<E>(self, v: &'v str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    self.visit_str(v)
                }
            }

            de.deserialize_any(Visit)
        }
    };
}

bruh!(one_or_many_target_type, TargetType);
bruh!(one_or_many_string, String);

#[derive(macros::UnusedKeys, serde::Deserialize, Debug)]
pub struct Target {
    #[serde(rename = "type", deserialize_with = "one_or_many_target_type")]
    pub type_: Vec<TargetType>,

    #[serde(deserialize_with = "one_or_many_string")]
    pub sources: Vec<String>,

    #[serde(deserialize_with = "one_or_many_string", default)]
    pub headers: Vec<String>,

    #[serde(deserialize_with = "one_or_many_string", default)]
    pub needs: Vec<String>,

    #[serde(flatten)]
    #[unused]
    pub rest: HashMap<String, toml::Value>,
}

impl Project {
    fn unique_targets_in_order_from<'my>(
        &'my self,
        target_name: &'my str,
        built: &mut HashSet<&'my str>,
    ) -> Result<Vec<(&'my str, &'my Target)>, Error> {
        let target = self
            .target
            .get(target_name)
            .ok_or_else(|| Error::NoSuchBuildTarget(target_name.into()))?;

        let mut targets = Vec::new();

        for needs in target.needs.iter() {
            if built.insert(needs.as_str()) {
                tracing::trace!("Will build {}: needed by {}", needs, target_name);
                targets.push((
                    needs.as_str(),
                    self.target
                        .get(needs)
                        .ok_or_else(|| Error::NoSuchBuildTarget(needs.into()))?,
                ));
            } else {
                tracing::trace!("Already building {}, needed by {}", needs, target_name);
            }
        }

        if built.insert(target_name) {
            tracing::trace!("Will build {}", target_name);
            targets.push((target_name, target));
        } else {
            tracing::trace!("Already built {}", target_name);
        }

        Ok(targets)
    }

    pub fn targets_in_order_from<'my>(
        &'my self,
        target_names: impl Iterator<Item = &'my str>,
    ) -> Result<Vec<(&'my str, &'my Target)>, Error> {
        let mut built = HashSet::new();
        let mut targets = Vec::new();
        for target_name in target_names {
            targets.extend(self.unique_targets_in_order_from(target_name, &mut built)?);
        }
        Ok(targets)
    }

    pub fn targets_in_order(&self) -> Result<Vec<(&str, &Target)>, Error> {
        self.targets_in_order_from(self.target.keys().map(|name| name.as_str()))
    }
}
