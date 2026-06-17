use std::borrow::Cow;

use crate::util::{to_pascal_case, to_snake_case};

// TODO: special name as a sperarate branch.

/// To better distinguish the name (ident).
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Name<'a> {
    /// The name works globally, like `text` or `float`, etc.
    ///
    /// At any place we can directly refer to it as `String`.
    Global(Cow<'a, str>),
    /// The name is at top level, like `js-uint` or `ResultData`.
    ///
    /// From top level, we refer to it as `JsUint`,
    /// while in modle we refer to it as `super::JsUint`.
    Top(Cow<'a, str>),
    /// The name is at module level, like `session.New`.
    ///
    /// From top-level we refer to it as `session::New`.
    /// In same modue, `New`.
    /// In different module, `super::session::New`.
    Namespaced(Cow<'a, str>, Cow<'a, str>),
}

impl<'a> Name<'a> {
    pub fn parse(name: &'a str) -> Self {
        let splits: Vec<_> = name.split(".").collect();
        match splits[..] {
            [
                "any" | "text" | "int" | "uint" | "float" | "true" | "false" | "bool" | "uuid"
                | "null" | "number",
            ] => Self::Global(name.into()),
            [name] => {
                // `-0`, `-Infinity`
                if name.starts_with("-") {
                    Self::Global(name.into())
                } else {
                    Self::Top(name.into())
                }
            },
            [module, name] => Self::Namespaced(module.into(), name.into()),
            _ => panic!("WebDriver BiDi only has one module level"),
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Name::Global(cow) => cow == "null",
            _ => false,
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            Name::Global(cow) => cow == "text",
            _ => false,
        }
    }

    fn as_special(&self) -> Option<&str> {
        Some(match self {
            Name::Global(g) => match &g[..] {
                "any" => "serde_json::Value",
                "text" => "String",
                "int" => "i64",
                "uint" => "u64",
                "float" | "number" => "f64",
                "true" | "false" | "bool" => "bool",
                "uuid" => "uuid::Uuid",
                _ => return None,
            },
            _ => return None,
        })
    }

    pub fn to_symbol_name(&self, pos: Option<&str>) -> String {
        if let Some(special) = self.as_special() {
            return special.to_string();
        }
        match self {
            Name::Global(g) => to_pascal_case(g.chars()).collect(),
            Name::Top(t) => {
                let t = to_pascal_case(t.chars()).collect();
                match pos {
                    Some(_) => format!("super::{}", t),
                    None => t,
                }
            },
            Name::Namespaced(m, n) => {
                let m: String = to_snake_case(m.chars()).collect();
                let n = to_pascal_case(n.chars()).collect();
                match pos {
                    Some(p) => {
                        let p: String = to_snake_case(p.chars()).collect();
                        if p == m {
                            n
                        } else {
                            format!("super::{m}::{n}")
                        }
                    },
                    None => format!("{m}::{n}"),
                }
            },
        }
    }

    pub fn to_field_name(&self) -> String {
        match self {
            Name::Global(s) | Name::Top(s) | Name::Namespaced(_, s) => {
                to_snake_case(s.chars()).collect()
            },
        }
    }

    pub fn to_variant_name(&self) -> String {
        match self {
            Name::Global(s) if s == "-0" => "NegZero".into(),
            Name::Global(s) if s == "-Infinity" => "NegInfinity".into(),
            Name::Global(s) | Name::Top(s) | Name::Namespaced(_, s) => {
                to_pascal_case(s.chars()).collect()
            },
        }
    }

    pub fn pos(&self) -> Option<&str> {
        match self {
            Name::Namespaced(m, _) => Some(m),
            _ => None,
        }
    }

    pub fn to_module_name(&self) -> Option<String> {
        match self {
            Name::Namespaced(m, _) => Some(to_snake_case(m.chars()).collect()),
            _ => None,
        }
    }

    pub fn to_raw(&self) -> String {
        match self {
            Name::Global(cow) => cow.to_string(),
            Name::Top(cow) => cow.to_string(),
            Name::Namespaced(cow, cow1) => format!("{cow}.{cow1}"),
        }
    }
}

impl<'a> From<&'a str> for Name<'a> {
    fn from(value: &'a str) -> Self {
        Self::parse(value)
    }
}
