use std::{collections::HashMap, fmt::Write};

use indexmap::IndexMap;

use crate::{
    common::Name,
    patterns::{
        BarewordVariant, EnumVariant, FieldVariant, InlineVariant, RulePattern, VecVariant,
    },
    syntax::{Field, Fields, File, Item, ItemEnum, ItemStruct, ItemType, Module, Variant},
};

/// manual patch to handle clippy::large_enum_variant
/// as it is bad to embed a whole clippy in codegen
const LARGE_ENUMS_VARIANTS: &[&str] = &[
    "session.New",
    "Event",
    "CommandResponse",
    "ErrorResponse",
    "NetworkEvent",
    "session.NewResult",
    "ScriptResult",
    "StorageResult",
    "script.Message",
    "script.RemoteValue",
];

pub fn pattern_map_to_file<'a>(rule_map: IndexMap<Name<'a>, RulePattern<'a>>) -> File {
    let mut items: Vec<Item> = vec![];
    let mut modules = IndexMap::<String, Module>::new();

    let tag_index = build_tag_index(&rule_map);
    let recursive_map = find_struct_recursive(&rule_map);

    for (rule_name, rule) in rule_map.iter() {
        let mod_items = pattern_to_items(rule_name, rule, &tag_index, &recursive_map);
        for (module, mut item) in mod_items {
            match module {
                None => items.push(item),
                Some(module) => {
                    let module = modules.entry(module.clone()).or_insert(Module {
                        name: module,
                        items: vec![],
                    });
                    *item.indent_mut() = true;
                    module.items.push(item);
                },
            }
        }
    }

    File {
        items,
        modules: modules.into_values().collect(),
    }
}

const SERDE_FLATTEN: &str = "#[serde(flatten)]";
const SERDE_SKIP: &str = "#[serde(skip_serializing_if = \"Option::is_none\")]";
const SERDE_UNTAGGED: &str = "#[serde(untagged)]";

pub fn pattern_to_items<'a>(
    item_name: &Name<'a>,
    pattern: &RulePattern<'a>,
    tag_index: &'a HashMap<&'a Name<'a>, TagStrategy>,
    recusrive_map: &'a HashMap<&'a Name<'a>, Vec<&'a Name<'a>>>,
) -> Vec<(Option<String>, Item)> {
    let pos = item_name.pos();
    let module = item_name.to_module_name();
    let tag = tag_index.get(item_name);
    let recursive = recusrive_map.get(item_name);
    let mut mod_items: Vec<(Option<String>, Item)> = vec![];

    match pattern {
        RulePattern::Struct(field_variants) => struct_pattern_to_item(
            field_variants,
            &mut mod_items,
            item_name,
            pos,
            module,
            tag,
            recursive,
        ),
        RulePattern::Enum(enum_variants) => {
            let mut variants = vec![];
            for enum_variant in enum_variants {
                match enum_variant {
                    // like `"foo" / "bar"`
                    EnumVariant::Literal(name) => variants.push(Variant::new_with_attrs(
                        name.to_variant_name(),
                        Fields::Unit,
                        vec![format!("#[serde(rename = \"{}\")]", name.to_raw())],
                    )),
                    // like `Foo // Bar`
                    EnumVariant::Ident(name) => {
                        let mut s_name = name.to_symbol_name(pos);
                        // handle large enum variant
                        if LARGE_ENUMS_VARIANTS.contains(&name.to_raw().as_str()) {
                            s_name = format!("Box<{s_name}>");
                        }
                        // handle single case
                        let fields = Fields::Unnamed(vec![s_name]);
                        variants.push(
                            Variant::new(name.to_variant_name(), fields)
                                .with_attr_if(tag.and_then(|t| t.to_variant_attr(name))),
                        );
                    },
                }
            }
            // sort to ensure untagged field appear last
            variants.sort_by_key(|v| v.attrs.iter().any(|a| a.contains("untagged")));

            mod_items.push((
                module,
                Item::Enum(
                    ItemEnum::new(item_name.to_variant_name(), variants)
                        .with_attr_if(tag.map(TagStrategy::to_enum_attr)),
                ),
            ));
        },
        RulePattern::Alias(ty) => {
            mod_items.push((
                module,
                Item::Type(ItemType {
                    indent: false,
                    name: item_name.to_variant_name(),
                    ty: ty.to_symbol_name(pos),
                }),
            ));
        },
        RulePattern::Vec(vec_variant) => match vec_variant {
            VecVariant::Ident(item_ty) => {
                mod_items.push((
                    module,
                    Item::Type(ItemType {
                        indent: false,
                        name: item_name.to_variant_name(),
                        ty: format!("Vec<{}>", item_ty.to_symbol_name(pos)),
                    }),
                ));
            },
            VecVariant::InlineChoice(_) => unreachable!("Currently no top level inline choice vec"),
            VecVariant::Nested(vec_variants) => {
                let mut s = "Vec<(".to_string();
                for v in vec_variants {
                    match v {
                        VecVariant::Ident(name) => {
                            write!(s, "{}, ", name.to_symbol_name(pos)).unwrap();
                        },
                        // like `[(js-uint / text)]`
                        VecVariant::InlineChoice(cows) => {
                            // derive a new enum for the inline choices
                            let inline_name = cows
                                .iter()
                                .map(|s| s.to_variant_name())
                                .collect::<Vec<_>>()
                                .join("Or");
                            write!(s, "{}, ", inline_name).unwrap();
                            mod_items.push((
                                module.clone(),
                                Item::Enum(ItemEnum::new(
                                    inline_name,
                                    cows.iter()
                                        .map(|c| {
                                            let mut ty = c.to_symbol_name(pos);
                                            if LARGE_ENUMS_VARIANTS.contains(&c.to_raw().as_str()) {
                                                ty = format!("Box<{ty}>");
                                            }
                                            Variant::new_with_attrs(
                                                c.to_variant_name(),
                                                Fields::Unnamed(vec![ty]),
                                                vec![SERDE_UNTAGGED.into()],
                                            )
                                        })
                                        .collect(),
                                )),
                            ));
                        },
                        VecVariant::Nested(_) => {
                            unreachable!("nested inline vec is not supported yet")
                        },
                    }
                }
                write!(s, ")>").unwrap();

                mod_items.push((
                    module,
                    Item::Type(ItemType {
                        indent: false,
                        name: item_name.to_symbol_name(pos),
                        ty: s,
                    }),
                ));
            },
        },
        RulePattern::HashMap(value_ty) => {
            mod_items.push((
                module,
                Item::Type(ItemType {
                    indent: false,
                    name: item_name.to_variant_name(),
                    ty: format!(
                        "std::collections::HashMap<String, {}>",
                        value_ty.to_symbol_name(pos)
                    ),
                }),
            ));
        },
    }

    mod_items
}

#[derive(Debug, Clone)]
pub enum TagStrategy {
    Tagged {
        field: String,
        renames: HashMap<String, String>,
        // There can be at most one fallback variant like `type: text`
        fallback: Option<String>,
        // though a enum is tagged, it may include other
        nested: Vec<String>,
    },
    Untagged,
}

impl TagStrategy {
    pub fn to_enum_attr(&self) -> String {
        match self {
            TagStrategy::Tagged { field, .. } => {
                format!("#[serde(tag = \"{}\")]", field)
            },
            TagStrategy::Untagged => SERDE_UNTAGGED.into(),
        }
    }

    pub fn to_variant_attr(&self, variant_name: &Name) -> Option<String> {
        match self {
            TagStrategy::Tagged {
                renames,
                fallback,
                nested,
                ..
            } => {
                let raw = variant_name.to_raw();

                // the nested enum
                if nested.contains(&raw) {
                    return Some(SERDE_UNTAGGED.into());
                }

                // the `text` fallback
                if let Some(fallback) = fallback
                    && fallback == &raw
                {
                    return Some(SERDE_UNTAGGED.into());
                }
                renames
                    .get(&variant_name.to_raw())
                    .map(|s| format!("#[serde(rename = \"{}\")]", s))
            },
            TagStrategy::Untagged => None,
        }
    }
}

/// We need tag information in advance to generate correct serde tag.
fn build_tag_index<'a>(
    rules: &'a IndexMap<Name<'a>, RulePattern<'a>>,
) -> HashMap<&'a Name<'a>, TagStrategy> {
    let mut tag_index = HashMap::new();

    for (rule_name, rule) in rules.iter() {
        // as serde tag only applies to enum, we only need to check enums.
        let RulePattern::Enum(enum_variants) = rule else {
            continue;
        };

        // collect all variant sub_names
        // currently only handle when each enum variant is ident.
        // not sure if this is the only case, but absolutely their handling is different.
        // e.g. literal => may be directly added without tag, field => different naming.
        let mut sub_names = vec![];
        for enum_variant in enum_variants {
            match enum_variant {
                EnumVariant::Ident(name) => {
                    sub_names.push(name);
                },
                // Therotically literal should not have tags,
                // but not sure if there is mixed ident and literal
                EnumVariant::Literal(_) => {},
            }
        }

        // goes here if all enum variant literal
        if sub_names.is_empty() {
            continue;
        }

        // actual checks
        for check in [check_enum_variants] {
            if let Some(tag) = check(&sub_names, rules) {
                // add tag strategy to container type
                tag_index.insert(rule_name, tag.clone());
                // we also add tag strategy to sub type, so that sub struct can hide the field used by tag.
                for sub_name in &sub_names {
                    tag_index.insert(sub_name, tag.clone());
                }
            }
        }
    }

    tag_index
}

// each sub type should return a [key, value][],
// then we find first intersection, and the outcome
// is actual field name like `proxyType`, along with
// `sub name` -> `rename` hashmap.
//
// if common field does not exists, fallback to untagged.
fn check_enum_variants<'a>(
    sub_names: &'a Vec<&'a Name<'a>>,
    rules: &'a IndexMap<Name<'a>, RulePattern<'a>>,
) -> Option<TagStrategy> {
    // map from field name to [sub_name, field_value][].
    // use index map to keep order, in case when there is multiple, the first is preferred.
    let mut candidates = IndexMap::<String, Vec<(String, Option<String>)>>::new();
    // this field record the sub that refer to a group.
    let mut nested = Vec::new();

    for sub_name in sub_names {
        let sub = rules.get(*sub_name).unwrap();
        match sub {
            RulePattern::Struct(fields) => {
                check_variant_struct_recursively(sub_name, fields, rules, &mut candidates);
            },
            RulePattern::Enum(_) => {
                nested.push(sub_name.to_raw());
            },
            _ => return None,
        }
    }

    for (field_name, matches) in candidates {
        // candidate field name should appear in all variant sub struct
        // but can be larger since one sub can have a choices of candidate.
        //
        // Actually `matches.len() + nested.len() < sub_names.len()` should be used,
        // but some may not have same tag
        if matches.len() < 2 {
            continue;
        }
        // candidate field should have at most one fallback (text) untagged branch.
        let mut fallback = None;
        for (sub_name, value) in &matches {
            if value.is_none() {
                if fallback.is_some() {
                    continue;
                }
                fallback = Some(sub_name.clone());
            }
        }
        // if all that matches, it is the candidate we want
        return Some(TagStrategy::Tagged {
            field: field_name,
            renames: matches
                .into_iter()
                .filter_map(|m| Some((m.0, m.1?)))
                .collect(),
            nested,
            fallback,
        });
    }

    // fallback to untagged
    Some(TagStrategy::Untagged)
}

pub fn struct_pattern_to_item<'a>(
    field_variants: &'a Vec<FieldVariant<'a>>,
    mod_items: &mut Vec<(Option<String>, Item)>,
    item_name: &Name<'a>,
    pos: Option<&'a str>,
    module: Option<String>,
    tag: Option<&'a TagStrategy>,
    recursive: Option<&'a Vec<&'a Name<'a>>>,
) {
    let mut fields = vec![];
    for field_variant in field_variants {
        match field_variant {
            FieldVariant::Bareword(skip, field_name, ty) => {
                // if this field is used as enum tag, skip it
                if let Some(TagStrategy::Tagged { field, .. }) = tag
                    && field == &field_name.to_raw()
                {
                    continue;
                }

                let rename = {
                    let raw = field_name.to_raw();
                    (field_name.to_field_name() != raw)
                        .then_some(format!("#[serde(rename = \"{}\")]", raw))
                };

                let mut ty_string: String = match ty {
                    // ordinary type name
                    BarewordVariant::Ident(n) => n.to_symbol_name(pos),
                    // vec like `[]`
                    BarewordVariant::Vec(vec_variant) => match vec_variant {
                        VecVariant::Ident(n) => format!("Vec<{}>", n.to_symbol_name(pos)),
                        VecVariant::InlineChoice(cows) => {
                            let inline_name = format!(
                                "{}{}Item",
                                item_name.to_variant_name(),
                                field_name.to_variant_name()
                            );
                            mod_items.push((
                                module.clone(),
                                Item::Enum(ItemEnum::new(
                                    inline_name.clone(),
                                    cows.iter()
                                        .map(|c| {
                                            let mut ty = c.to_symbol_name(pos);
                                            if LARGE_ENUMS_VARIANTS.contains(&c.to_raw().as_str()) {
                                                ty = format!("Box<{ty}>");
                                            }
                                            Variant::new_with_attrs(
                                                c.to_variant_name(),
                                                Fields::Unnamed(vec![ty]),
                                                vec![SERDE_UNTAGGED.into()],
                                            )
                                        })
                                        .collect(),
                                )),
                            ));
                            inline_name
                        },
                        VecVariant::Nested(_) => unreachable!(),
                    },
                    // enum should be handled by. If not handled, fallback to inline enum here.
                    BarewordVariant::Enum(enum_variants) => {
                        let end_with_null = matches!(enum_variants.last(), Some(EnumVariant::Ident(il)) if il.is_null());
                        // if is single except for last null => Option single
                        if end_with_null && let [EnumVariant::Ident(i0), _] = &enum_variants[..] {
                            let s_name = i0.to_symbol_name(pos);
                            if let Some(recursive) = recursive
                                && recursive.contains(&i0)
                            {
                                format!("Option<Box<{}>>", s_name)
                            } else {
                                format!("Option<{}>", i0.to_symbol_name(pos))
                            }
                        } else {
                            // inline name by concat item enum name and the field name,
                            // e.g.
                            // browser.ClientWindowInfo = { state: "fullscreen" / "maximized" }
                            // to ClientWindowInfoState
                            let inline_name = format!(
                                "{}{}",
                                item_name.to_variant_name(),
                                field_name.to_variant_name(),
                            );
                            let mut vs = vec![];
                            for (idx, x) in enum_variants.iter().enumerate() {
                                // if end with null, skip last null
                                if end_with_null && idx == enum_variants.len() - 1 {
                                    continue;
                                }
                                vs.push(match x {
                                    EnumVariant::Literal(name) => Variant::new_with_attrs(
                                        name.to_variant_name(),
                                        Fields::Unit,
                                        vec![format!("#[serde(rename = \"{}\")]", name.to_raw())],
                                    ),
                                    EnumVariant::Ident(name) => Variant::new_with_attrs(
                                        name.to_variant_name(),
                                        Fields::Unnamed(vec![name.to_symbol_name(pos)]),
                                        vec![format!("#[serde(rename = \"{}\")]", name.to_raw())],
                                    ),
                                });
                            }

                            mod_items.push((
                                module.clone(),
                                Item::Enum(ItemEnum::new(inline_name.clone(), vs)),
                            ));

                            if end_with_null {
                                format!("Option<{}>", inline_name)
                            } else {
                                inline_name.to_string()
                            }
                        }
                    },
                    BarewordVariant::InlineStruct(items) => {
                        let inline_name = format!(
                            "{}{}",
                            item_name.to_variant_name(),
                            field_name.to_variant_name(),
                        );
                        struct_pattern_to_item(
                            items,
                            mod_items,
                            &inline_name.as_str().into(),
                            pos,
                            module.clone(),
                            tag,
                            recursive,
                        );
                        inline_name
                    },
                    BarewordVariant::HashMap(n) => format!(
                        "std::collections::HashMap<String, {}>",
                        n.to_symbol_name(pos)
                    ),
                };

                // some field may be optional but null choice missing
                // like `browsingContext.Info["children"]`
                if *skip && !ty_string.starts_with("Option<") {
                    ty_string = format!("Option<{ty_string}>");
                }
                fields.push(
                    Field::new(field_name.to_field_name(), ty_string)
                        .with_attr_if(skip.then(|| SERDE_SKIP.into()))
                        .with_attr_if(rename),
                );
            },
            FieldVariant::Flatten(ty) => {
                let field_name = ty.to_field_name();
                fields.push(Field::new_with_attrs(
                    field_name,
                    ty.to_symbol_name(pos),
                    vec![SERDE_FLATTEN.into()],
                ));
            },
            FieldVariant::Inline(inline_variant) => {
                match inline_variant {
                    InlineVariant::Idents(cows) => {
                        let inline_symbol_name = cows
                            .iter()
                            .map(|n| n.to_variant_name())
                            .collect::<Vec<_>>()
                            .join("Or");
                        let bad_inline_field_name = cows
                            .iter()
                            .map(|n| n.to_field_name())
                            .collect::<Vec<_>>()
                            .join("_or_");

                        // add to field
                        fields.push(Field::new_with_attrs(
                            bad_inline_field_name,
                            inline_symbol_name.clone(),
                            vec![SERDE_FLATTEN.into()],
                        ));

                        // add derived item
                        mod_items.push((
                            module.clone(),
                            Item::Enum(ItemEnum::new(
                                inline_symbol_name,
                                cows.iter()
                                    .map(|s| {
                                        Variant::new(
                                            s.to_variant_name(),
                                            Fields::Unnamed(vec![s.to_symbol_name(pos)]),
                                        )
                                    })
                                    .collect(),
                            )),
                        ));
                    },
                    // Like `Foo = { (name: text) // (value: int) }`
                    InlineVariant::Fields(items) => {
                        let field_name = items
                            .iter()
                            .map(|n| n.0.to_field_name())
                            .collect::<Vec<_>>()
                            .join("_or_");
                        let symbol_name = item_name.to_variant_name()
                            + &items
                                .iter()
                                .map(|n| n.0.to_variant_name())
                                .collect::<Vec<_>>()
                                .join("Or");

                        // add to field
                        fields.push(Field::new_with_attrs(
                            field_name,
                            symbol_name.clone(),
                            vec![SERDE_FLATTEN.into()],
                        ));

                        let mut vs = vec![];
                        for (f_name, type_choices) in items {
                            let ty = match &type_choices[..] {
                                [single] => single.to_symbol_name(pos),
                                [t0, t1] if t1.is_null() => {
                                    format!("Option<{}>", t0.to_symbol_name(pos))
                                },
                                _ => unreachable!("complex type choices in inline field"),
                            };
                            vs.push(Variant::new_with_attrs(
                                f_name.to_variant_name(),
                                Fields::Named(vec![(f_name.to_field_name(), ty)]),
                                vec![format!("#[serde(rename = \"{}\")]", f_name.to_raw())],
                            ));
                        }

                        // add derived item
                        mod_items
                            .push((module.clone(), Item::Enum(ItemEnum::new(symbol_name, vs))));
                    },
                }
            },
        }
    }
    mod_items.push((
        module,
        Item::Struct(ItemStruct {
            indent: false,
            name: item_name.to_variant_name(),
            fields,
        }),
    ));
}

// Since some struct can have inline group fields, we need to check it recursively.
fn check_variant_struct_recursively<'a>(
    sub_name: &'a &'a Name,
    fields: &Vec<FieldVariant>,
    rules: &'a IndexMap<Name<'a>, RulePattern<'a>>,
    candidates: &mut IndexMap<String, Vec<(String, Option<String>)>>,
) {
    for field in fields {
        match field {
            FieldVariant::Bareword(_, field_name, ty) => {
                // when the field is `type: "foo"` or `type: "foo" / "bar"`
                // both single literal and literal choices are allowed
                // check all choice is literal
                if let BarewordVariant::Enum(vs) = &ty
                    && let Some(ls) = vs
                        .iter()
                        .map(|v| match v {
                            EnumVariant::Literal(name) => Some(name),
                            _ => None,
                        })
                        .collect::<Option<Vec<_>>>()
                {
                    for l in ls {
                        candidates
                            .entry(field_name.to_raw())
                            .or_default()
                            .push((sub_name.to_raw(), Some(l.to_raw())));
                    }
                }
                // handle special case when all others are literals when one be text and fallback
                // See `browser.ClientWindowNamedState`
                if let BarewordVariant::Ident(i) = &ty
                    && i.is_text()
                {
                    candidates
                        .entry(field_name.to_raw())
                        .or_default()
                        .push((sub_name.to_raw(), None));
                }
            },
            // Like `script.DateRemoteValue = { script.DateLocalValue, }`
            // we also need to check datelocalvalue
            FieldVariant::Flatten(sub_sub_name) => {
                let subsub = rules.get(sub_sub_name).unwrap();
                if let RulePattern::Struct(subsub_variants) = subsub {
                    check_variant_struct_recursively(
                        // use original subname instead of subsub,
                        // because we will match against original branch name
                        sub_name,
                        subsub_variants,
                        rules,
                        candidates,
                    )
                }
            },
            // inline is unrelavant but should cause a None
            FieldVariant::Inline(_) => {},
        }
    }
}

// XXX: slow, adhoc and incomplete.
// should we use dfs? but loc already bloats, for a build.rs
// and we already know that's not happening in spec.
fn find_struct_recursive<'a>(
    rules: &'a IndexMap<Name<'a>, RulePattern<'a>>,
) -> HashMap<&'a Name<'a>, Vec<&'a Name<'a>>> {
    let mut map = HashMap::<&'a Name<'a>, Vec<&'a Name<'a>>>::new();
    for name in rules.keys() {
        let deps = find_struct_direct_deps(rules, name);
        for dep in deps {
            let dep_dep = find_struct_direct_deps(rules, dep);
            if dep_dep.contains(&name) {
                map.entry(name).or_default().push(dep);
            }
        }
    }
    map
}

fn find_struct_direct_deps<'a>(
    rules: &'a IndexMap<Name<'a>, RulePattern<'a>>,
    rule_name: &'a Name<'a>,
) -> Vec<&'a Name<'a>> {
    let mut deps = vec![];
    let Some(rule) = rules.get(rule_name) else {
        return deps;
    };
    if let RulePattern::Struct(field_variants) = rule {
        for field in field_variants {
            if let FieldVariant::Bareword(_, _, bareword_variant) = field {
                match bareword_variant {
                    BarewordVariant::Ident(name) => deps.push(name),
                    BarewordVariant::Enum(enum_variants) => {
                        for ev in enum_variants {
                            if let EnumVariant::Ident(name) = ev {
                                deps.push(name);
                            }
                        }
                    },
                    _ => {},
                }
            }
        }
    }
    deps
}
