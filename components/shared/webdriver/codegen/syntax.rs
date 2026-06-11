//! A simplified representation of Rust syntax, used for code generation.

use std::fmt::{self, Display, Formatter};

use super::util::is_rust_keyword;

pub struct File {
    /// top level items.
    pub items: Vec<Item>,
    pub modules: Vec<Module>,
}

impl File {
    pub fn merge(&mut self, mut other: Self) {
        self.items.append(&mut other.items);
        for mut om in other.modules {
            if let Some(m) = self.modules.iter_mut().find(|m| m.name == om.name) {
                m.items.append(&mut om.items);
            } else {
                self.modules.push(om);
            }
        }
        // dedup
        self.items.sort();
        self.items.dedup();
        self.modules.iter_mut().for_each(|m| {
            m.items.sort();
            m.items.dedup()
        });
    }
}

pub struct Module {
    pub name: String,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub enum Item {
    Enum(ItemEnum),
    Struct(ItemStruct),
    Type(ItemType),
}

impl Item {
    pub fn name(&self) -> &str {
        match self {
            Item::Enum(i) => &i.name,
            Item::Struct(i) => &i.name,
            Item::Type(i) => &i.name,
        }
    }

    pub fn indent_mut(&mut self) -> &mut bool {
        match self {
            Item::Enum(i) => &mut i.indent,
            Item::Struct(i) => &mut i.indent,
            Item::Type(i) => &mut i.indent,
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for Item {}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name().cmp(other.name())
    }
}

#[derive(Clone, Debug)]
pub struct ItemEnum {
    pub name: String,
    pub variants: Vec<Variant>,
    pub indent: bool,
    pub attrs: Vec<String>,
}

impl ItemEnum {
    pub fn new(name: String, variants: Vec<Variant>) -> Self {
        Self::new_with_attrs(name, variants, vec![])
    }

    pub fn new_with_attrs(name: String, variants: Vec<Variant>, attrs: Vec<String>) -> Self {
        Self {
            name,
            variants,
            indent: false,
            attrs,
        }
    }

    pub fn with_attr(mut self, attr: String) -> Self {
        self.attrs.push(attr);
        self
    }

    pub fn with_attr_if(self, attr: Option<String>) -> Self {
        if let Some(attr) = attr {
            self.with_attr(attr)
        } else {
            self
        }
    }
}

#[derive(Clone, Debug)]
pub struct ItemStruct {
    pub name: String,
    pub fields: Vec<Field>,
    pub indent: bool,
}

#[derive(Clone, Debug)]
pub struct ItemType {
    pub indent: bool,
    pub name: String,
    pub ty: String,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub ty: String,
    pub attrs: Vec<String>,
}

impl Field {
    pub fn new(name: String, ty: String) -> Self {
        Self::new_with_attrs(name, ty, vec![])
    }

    pub fn new_with_attrs(name: String, ty: String, attrs: Vec<String>) -> Self {
        Self { name, ty, attrs }
    }

    pub fn with_attr(mut self, attr: String) -> Self {
        self.attrs.push(attr);
        self
    }

    pub fn with_attr_if(self, attr: Option<String>) -> Self {
        if let Some(attr) = attr {
            self.with_attr(attr)
        } else {
            self
        }
    }
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub name: String,
    pub fields: Fields,
    pub attrs: Vec<String>,
}

impl Variant {
    pub fn new(name: String, fields: Fields) -> Self {
        Self::new_with_attrs(name, fields, vec![])
    }

    pub fn new_with_attrs(name: String, fields: Fields, attrs: Vec<String>) -> Self {
        Self {
            name,
            fields,
            attrs,
        }
    }

    pub fn with_attr(mut self, attr: String) -> Self {
        self.attrs.push(attr);
        self
    }

    pub fn with_attr_if(self, attr: Option<String>) -> Self {
        if let Some(attr) = attr {
            self.with_attr(attr)
        } else {
            self
        }
    }
}

#[derive(Clone, Debug)]
pub enum Fields {
    Named(Vec<(String, String)>),
    Unnamed(Vec<String>),
    Unit,
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // top level items
        for item in &self.items {
            item.fmt(f)?;
        }
        // modules
        for module in &self.modules {
            module.fmt(f)?;
        }
        Ok(())
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "pub mod {} {{", self.name)?;
        for item in &self.items {
            item.fmt(f)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Item::Enum(item_enum) => item_enum.fmt(f),
            Item::Struct(item_struct) => item_struct.fmt(f),
            Item::Type(item_type) => item_type.fmt(f),
        }
    }
}

const INDENT: &str = "    ";

impl Display for ItemEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let indent_mod = if self.indent { INDENT } else { "" };
        writeln!(
            f,
            "{indent_mod}#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]"
        )?;
        for attr in &self.attrs {
            writeln!(f, "{indent_mod}{attr}")?;
        }
        writeln!(f, "{indent_mod}pub enum {} {{", self.name)?;
        for variant in &self.variants {
            for attr in &variant.attrs {
                writeln!(f, "{indent_mod}{INDENT}{attr}")?;
            }
            match &variant.fields {
                Fields::Named(items) => writeln!(
                    f,
                    "{indent_mod}{INDENT}{} {{ {} }},",
                    variant.name,
                    items
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Fields::Unnamed(items) => {
                    writeln!(
                        f,
                        "{indent_mod}{INDENT}{}({}),",
                        variant.name,
                        items.join(", ")
                    )
                },
                Fields::Unit => writeln!(f, "{indent_mod}{INDENT}{},", variant.name),
            }?
        }
        writeln!(f, "{indent_mod}}}")?;
        Ok(())
    }
}

impl Display for ItemStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let indent_mod = if self.indent { INDENT } else { "" };
        writeln!(
            f,
            "{indent_mod}#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]"
        )?;
        writeln!(f, "{indent_mod}pub struct {} {{", self.name)?;
        for field in &self.fields {
            for attr in &field.attrs {
                writeln!(f, "{indent_mod}{INDENT}{attr}")?;
            }
            let field_name = if is_rust_keyword(&field.name) {
                format_args!("r#{}", field.name)
            } else {
                format_args!("{}", field.name)
            };
            writeln!(f, "{indent_mod}{INDENT}{}: {},", field_name, field.ty)?;
        }
        writeln!(f, "{indent_mod}}}")?;
        Ok(())
    }
}

impl Display for ItemType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let indent_mod = if self.indent { INDENT } else { "" };
        writeln!(f, "{indent_mod}pub type {} = {};", self.name, self.ty)
    }
}
