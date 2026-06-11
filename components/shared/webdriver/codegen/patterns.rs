use cddl::ast::{
    CDDL, Group, GroupChoice, GroupEntry, MemberKey, Occur, OptionalComma, Rule, Type, Type1,
    Type2, TypeChoice, ValueMemberKeyEntry,
};
use indexmap::IndexMap;

use super::common::Name;

/// In first iteration we build index.
pub fn parse_into_patterns<'a>(
    cddl: &'a CDDL<'a>,
    debug: bool,
) -> IndexMap<Name<'a>, RulePattern<'a>> {
    let mut rule_map = IndexMap::new();

    for rule in &cddl.rules {
        if debug {
            println!("{:#?}", rule);
        }
        let name = match rule {
            Rule::Type { rule, .. } => rule.name.ident,
            Rule::Group { rule, .. } => rule.name.ident,
        };
        let rule = override_rule(rule)
            .or(RulePattern::parse(rule).ok())
            .unwrap();
        rule_map.insert(name.into(), rule);
    }

    rule_map
}

/// Fix some rules that is not in CDDL but specified in spec.
///
/// Currently the only use case is for text->UUID override.
fn override_rule<'a>(rule: &Rule<'a>) -> Option<RulePattern<'a>> {
    match &rule.name()[..] {
        "session.Subscription"
        | "browsingContext.BrowsingContext"
        | "browsingContext.Screencast"
        | "browsingContext.Download"
        | "network.Collector"
        | "network.Intercept"
        | "script.PreloadScript" => Some(RulePattern::Alias("uuid".into())),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum RulePattern<'a> {
    /// Structs.
    /// Like `Foo = { bar: Bar, Baz }`
    Struct(Vec<FieldVariant<'a>>),
    /// Enum of name of other types.
    /// Like `Foo = ( Bar // Baz // Blah )`.
    /// Or `Foo = Bar / Baz / Blah`.
    /// Or `Foo = { Bar // Baz // Blah }`
    /// Or string literals.
    /// Like `Foo = "bar" / "baz" / "blah"`.
    Enum(Vec<EnumVariant<'a>>),
    /// Like `Foo = Bar`.
    Alias(Name<'a>),
    // the following two may be merged into alias?
    /// Like `Foo = [*Bar]`.
    Vec(VecVariant<'a>),
    /// Like `Foo = (*text => any)`.
    /// Currently only text => _ is used.
    HashMap(Name<'a>),
}

impl<'a> RulePattern<'a> {
    /// Parse raw CDDL rule into known patterns.
    pub fn parse(rule: &'a Rule) -> Result<Self, String> {
        match rule {
            Rule::Type { rule, .. } => {
                if rule.is_type_choice_alternate {
                    return Err("type rule as choice alternative is not implemented".into());
                }
                let type_choices = &rule.value.type_choices;
                match &type_choices[..] {
                    [] => Err("empty type rule is not allowed in CDDL".into()),
                    // when there is not outmost `/`, this includes many cases: struct, enums, alias, etc.
                    [single] => {
                        let t2 = &single.type1.type2;
                        match t2 {
                            // number types
                            Type2::IntValue { .. } => Ok(Self::Alias("int".into())),
                            Type2::UintValue { .. } => Ok(Self::Alias("uint".into())),
                            Type2::FloatValue { .. } => Ok(Self::Alias("float".into())),
                            // single string literal
                            Type2::TextValue { value, .. } => Ok(Self::Enum(vec![EnumVariant::Literal(value.as_ref().into())])),
                            // alias as another type.
                            Type2::Typename { ident, .. } => Ok(Self::Alias(ident.ident.into())),
                            // alias as vec of another type group.
                            Type2::Array { group, .. } => Self::new_vec_from_group_choices(group),
                            // this is an indirection, like in `browsingContext.Locator`.
                            Type2::ParenthesizedType { pt, .. } => {
                                Self::new_enum_from_type_choices(&pt.type_choices)
                            },
                            Type2::Map { group, .. } => {
                                let group_choices = &group.group_choices;
                                match &group_choices[..] {
                                    [] => Err("group with empty choices is not allowed in CDDL".into()),
                                    [single] => Self::new_struct_from_group_entries(&single.group_entries),
                                    // there are many choices, like `Foo = { Bar // Baz }`.
                                    // used in `session.ProxyConfiguration`
                                    _ => Self::new_enum_from_group_choices(group_choices)
                                }
                            },
                            Type2::ChoiceFromInlineGroup { .. } => Err("choice from inline group is not implemented yet".into()),
                            Type2::ChoiceFromGroup { .. } => Err("choice from group is not implemented yet".into()),
                            Type2::UTF8ByteString { .. }
                            | Type2::B16ByteString { .. }
                            | Type2::B64ByteString { .. } => Err("byte strings are not implemented yet".into()),
                            Type2::Unwrap { .. }
                            | Type2::TaggedData { .. }
                            | Type2::DataMajorType {.. } => {
                                Err("advanced features like Unwrap, TaggedData, DataMajorType is not implemented yet".into())
                            },
                            Type2::Any { .. } => Err("any type is not implemented yet".into()),
                        }
                    },
                    // when there are many type choices, like `Foo = Bar / "blah"`
                    _ => Self::new_enum_from_type_choices(type_choices),
                }
            },
            Rule::Group { rule, .. } => match &rule.entry {
                GroupEntry::ValueMemberKey { .. } => Err(
                    "unparened value member key seems to be imposibble in top level group rule"
                        .into(),
                ),
                GroupEntry::TypeGroupname { .. } => Err(
                    "unparened type group name seem to be impossible in top level group rule"
                        .into(),
                ),
                // The normal group rule, like `Foo = ( Bar // Baz // Blah )`,
                // note that `( Bar / Baz / Blah )` is a type rule instead of group rule
                GroupEntry::InlineGroup { group, .. } => {
                    let group_choices = &group.group_choices;
                    match group_choices.len() {
                        0 => Err("empty group without choice is not permitted by CDDL".into()),
                        // Like `Foo = ( bar: Bar, Blah, text => any )`
                        1 => {
                            let entries = &group_choices[0].group_entries;
                            // there are 2 cases:
                            match &entries[..] {
                                // 1. hashmap like `(text => any)`, only one such is allowed
                                [(GroupEntry::ValueMemberKey { ge, .. }, _)]
                                    if let Some(MemberKey::Type1 { t1, .. }) = &ge.member_key =>
                                {
                                    Self::new_hashmap(t1, &ge.entry_type)
                                },
                                // 2. struct like `( foo: Foo, Bar )`, with each entry being a field.
                                _ => Self::new_struct_from_group_entries(entries),
                            }
                        },
                        // Like `Foo = ( Bar // ( blah: Blah ) )`.
                        //
                        // While currently we only support `Foo = ( Bar // Baz )`.
                        //
                        // Inline choice is possible but is not used in WebDriver,
                        // so we haven't implemented it yet.
                        2.. => Self::new_enum_from_group_choices(group_choices),
                    }
                },
            },
        }
    }

    fn new_enum_from_type_choices(type_choices: &'a Vec<TypeChoice<'a>>) -> Result<Self, String> {
        let mut variants = vec![];

        for type_choice in type_choices {
            match &type_choice.type1.type2 {
                Type2::IntValue { .. } | Type2::UintValue { .. } | Type2::FloatValue { .. } => {
                    return Err("numbers cannot be used as enum variant".into());
                },
                Type2::TextValue { value, .. } => {
                    variants.push(EnumVariant::Literal(value.as_ref().into()));
                },
                Type2::UTF8ByteString { .. }
                | Type2::B16ByteString { .. }
                | Type2::B64ByteString { .. } => {
                    return Err("byte strings cannot be used as enum variant".into());
                },
                Type2::Typename { ident, .. } => {
                    variants.push(EnumVariant::Ident(ident.ident.into()));
                },
                Type2::ParenthesizedType { .. } => {
                    return Err("paren in type choice is not implemented yet".into());
                },
                Type2::Map { group, .. } => {
                    if let [gc] = &group.group_choices[..]
                        && let [(GroupEntry::TypeGroupname { ge, .. }, _)] = &gc.group_entries[..]
                    {
                        variants.push(EnumVariant::Ident(ge.name.ident.into()));
                    } else {
                        return Err("only one direct group is implemented in map type".into());
                    }
                },
                Type2::Array { .. } => {
                    return Err("group in type choice is not allowed".into());
                },
                Type2::ChoiceFromInlineGroup { .. } => {
                    return Err("nested inline choice is not implemented yet".into());
                },
                Type2::ChoiceFromGroup { .. } => {
                    return Err("nested choice is not implemented yet".into());
                },
                Type2::Unwrap { .. } | Type2::TaggedData { .. } | Type2::DataMajorType { .. } => {
                    return Err("advanced feature like Unwrap, TaggedData is not allowed".into());
                },
                Type2::Any { .. } => {
                    return Err("any type is not allowed".into());
                },
            }
        }
        Ok(Self::Enum(variants))
    }

    /// Currently only `text => any` is supported.
    fn new_hashmap(t1: &Type1, entry_type: &'a Type) -> Result<Self, String> {
        if t1.operator.is_none()
            && let Type2::Typename { ident, .. } = &t1.type2
            && ident.ident == "text"
            && let [tc] = &entry_type.type_choices[..]
            && let Type2::Typename { ident, .. } = &tc.type1.type2
        {
            match ident.ident {
                "any" | "text" => return Ok(RulePattern::HashMap(ident.ident.into())),
                _ => {},
            }
        }
        Err("hashmap like other than `text => any` or `text => text` is not implemented yet".into())
    }

    fn new_struct_from_group_entries(
        entries: &'a Vec<(GroupEntry<'a>, OptionalComma<'a>)>,
    ) -> Result<Self, String> {
        let mut field_variants = vec![];
        // handle value, handle `=>`
        for (entry, _) in entries {
            match entry {
                GroupEntry::ValueMemberKey { ge, .. } => {
                    let skip_serializing = matches!(
                        ge.occur.as_ref().map(|o| o.occur),
                        Some(Occur::Optional { .. })
                    );
                    match &ge.member_key {
                        Some(MemberKey::Bareword { ident, .. }) => {
                            field_variants.push(FieldVariant::Bareword(
                                skip_serializing,
                                ident.ident.into(),
                                BarewordVariant::new_from_type(&ge.entry_type)?,
                            ));
                        },
                        // NOTE: is this an error in cddl parser?
                        // the attribuet itself should be bareword
                        Some(MemberKey::Type1 { t1, .. }) => {
                            if let Type2::Typename { ident, .. } = &t1.type2 {
                                field_variants.push(FieldVariant::Bareword(
                                    skip_serializing,
                                    ident.ident.into(),
                                    BarewordVariant::new_from_type(&ge.entry_type)?,
                                ));
                            } else {
                                return Err(format!("is this type1: {t1:?}"));
                            }
                        },
                        _ => {
                            return Err("only bareword is supported as struct field name".into());
                        },
                    }
                },
                GroupEntry::TypeGroupname { ge, .. } => {
                    field_variants.push(FieldVariant::Flatten(ge.name.ident.into()));
                },
                // inline group must be
                GroupEntry::InlineGroup { group, .. } => {
                    let mut inline_idents: Vec<Name<'a>> = vec![];
                    let mut inline_fields = vec![];
                    for gc in &group.group_choices {
                        if let [(ge, _)] = &gc.group_entries[..] {
                            match ge {
                                GroupEntry::ValueMemberKey { .. } => {
                                    return Err("value member is not allowed directly in struct inline group".into());
                                },
                                GroupEntry::TypeGroupname { ge, .. } => {
                                    inline_idents.push(ge.name.ident.into());
                                },
                                GroupEntry::InlineGroup { group, .. } => {
                                    // so here is immediately a child inline group, and the child group now
                                    // only permits value member. :(
                                    if let [gc] = &group.group_choices[..]
                                        && let [(GroupEntry::ValueMemberKey { ge, .. }, _)] =
                                            &gc.group_entries[..]
                                        && let Some(MemberKey::Bareword { ident, .. }) =
                                            &ge.member_key
                                        && let Some(type_choices) = ge
                                            .entry_type
                                            .type_choices
                                            .iter()
                                            .map(|tc| {
                                                if let Type2::Typename { ident, .. } =
                                                    &tc.type1.type2
                                                {
                                                    Some(ident.ident.into())
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect()
                                    {
                                        inline_fields.push((ident.ident.into(), type_choices))
                                    } else {
                                        return Err("in inner inline group of struct inline group, only one field is used".into());
                                    }
                                },
                            }
                        } else {
                            return Err(
                                "in struct inline group, each group entry should have exactly one entry"
                                    .into(),
                            );
                        }
                    }
                    match (inline_idents.is_empty(), inline_fields.is_empty()) {
                        (true, false) => {
                            field_variants
                                .push(FieldVariant::Inline(InlineVariant::Fields(inline_fields)));
                        },
                        (false, true) => {
                            field_variants
                                .push(FieldVariant::Inline(InlineVariant::Idents(inline_idents)));
                        },
                        _ => {
                            return Err(
                                "struct inline group has to be either all idents or all fields"
                                    .into(),
                            );
                        },
                    }
                },
            }
        }
        Ok(RulePattern::Struct(field_variants))
    }

    fn new_enum_from_group_choices(
        group_choices: &'a Vec<GroupChoice<'a>>,
    ) -> Result<Self, String> {
        group_choices
            .iter()
            .map(|gc| match &gc.group_entries[..] {
                // the named group choices that we support
                [(GroupEntry::TypeGroupname { ge, .. }, _)] => {
                    Ok(EnumVariant::Ident(ge.name.ident.into()))
                },
                // parened or unparened inline choice
                _ => Err("group rule choices with inline group is not implemented yet".into()),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(RulePattern::Enum)
    }

    fn new_vec_from_group_choices(group: &'a Group<'a>) -> Result<Self, String> {
        match &group.group_choices[..] {
            [] => Err("group with empty choices is invalid in CDDL".into()),
            [gc] => {
                let group_entries = &gc.group_entries;
                match &group_entries[..] {
                    [] => Err("empty array is not supported".into()),
                    [(ge, _)] => {
                        match &ge {
                            // XXX: this branch is a bit hardcoded
                            GroupEntry::ValueMemberKey { ge, .. } => {
                                VecVariant::new_from_ge(ge).map(Self::Vec)
                            },
                            GroupEntry::TypeGroupname { ge, .. } => {
                                Ok(Self::Vec(VecVariant::Ident(ge.name.ident.into())))
                            },
                            _ => Err("inline group is not supported in array rule".into()),
                        }
                    },
                    _ => Err("multiple group entries is not supported in array rule".into()),
                }
            },
            // many group choices
            _ => Err("group choices in array is not supported".into()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EnumVariant<'a> {
    Literal(Name<'a>),
    Ident(Name<'a>),
}

#[derive(Debug, Clone)]
pub enum VecVariant<'a> {
    /// Like `[+browser.UserContext]`
    Ident(Name<'a>),
    // inline liternal vec not handled
    /// Like `[*(js-uint / text)]`
    InlineChoice(Vec<Name<'a>>),
    /// Like in `script.MappingLocalValue = [*[(script.LocalValue / text), script.LocalValue]];`
    Nested(Vec<VecVariant<'a>>),
}

impl<'a> VecVariant<'a> {
    fn new_from_ge(ge: &'a ValueMemberKeyEntry<'a>) -> Result<Self, String> {
        if let [tc] = &ge.entry_type.type_choices[..]
            && let Type2::Array { group, .. } = &tc.type1.type2
            && let [gc] = &group.group_choices[..]
        {
            let mut variants = vec![];
            for (ge, _) in &gc.group_entries {
                match ge {
                    GroupEntry::ValueMemberKey { .. } => {
                        return Err("value member not allowed for nested vec variant".into());
                    },
                    GroupEntry::TypeGroupname { ge, .. } => {
                        variants.push(VecVariant::Ident(ge.name.ident.into()));
                    },
                    GroupEntry::InlineGroup { group, .. } => {
                        if let [gc] = &group.group_choices[..]
                            && let [(GroupEntry::ValueMemberKey { ge, .. }, _)] =
                                &gc.group_entries[..]
                        {
                            let mut choices = vec![];
                            for tc in &ge.entry_type.type_choices {
                                if let Type2::Typename { ident, .. } = &tc.type1.type2 {
                                    choices.push(ident.ident.into());
                                } else {
                                    return Err(
                                        "only type name is allowed in nested vec type choice"
                                            .into(),
                                    );
                                }
                            }
                            variants.push(VecVariant::InlineChoice(choices));
                        } else {
                            return Err("only type choice is allowed in nested vec".into());
                        }
                    },
                }
            }
            Ok(VecVariant::Nested(variants))
        } else {
            Err("others nested array syntax not implemented".into())
        }
    }
}

#[derive(Debug, Clone)]
pub enum InlineVariant<'a> {
    Idents(Vec<Name<'a>>),
    Fields(Vec<(Name<'a>, Vec<Name<'a>>)>),
}

#[derive(Debug, Clone)]
pub enum BarewordVariant<'a> {
    Ident(Name<'a>),
    Vec(VecVariant<'a>),
    // include single literal and nullable
    Enum(Vec<EnumVariant<'a>>),
    InlineStruct(Vec<FieldVariant<'a>>),
    HashMap(Name<'a>),
}

impl<'a> BarewordVariant<'a> {
    fn new_from_type(ty: &'a Type<'a>) -> Result<Self, String> {
        let type_choices = &ty.type_choices;
        match &type_choices[..] {
            [] => Err("type with empty choices is invalid in CDDL".into()),
            [single] => {
                match &single.type1.type2 {
                    // numbers
                    Type2::IntValue { .. } => Ok(Self::Ident("int".into())),
                    Type2::UintValue { .. } => Ok(Self::Ident("uint".into())),
                    Type2::FloatValue { .. } => Ok(Self::Ident("float".into())),
                    // single literal
                    Type2::TextValue { value, .. } => Ok(Self::Enum(vec![EnumVariant::Literal(
                        value.as_ref().into(),
                    )])),
                    Type2::Typename { ident, .. } => Ok(Self::Ident(ident.ident.into())),
                    Type2::ParenthesizedType { pt, .. } => match &pt.type_choices[..] {
                        [] => Err("empty type choice invalid in CDDL".into()),
                        // for sinle, can be ident, enum, or etc
                        [tc] => {
                            match &tc.type1.type2 {
                                Type2::IntValue { .. } => Ok(Self::Ident("int".into())),
                                Type2::UintValue { .. } => Ok(Self::Ident("uint".into())),
                                Type2::FloatValue { .. } => Ok(Self::Ident("float".into())),
                                // single string literal is still viewed as enum litera;
                                Type2::TextValue { value, .. } => {
                                    Ok(Self::Enum(vec![EnumVariant::Literal(
                                        value.as_ref().into(),
                                    )]))
                                },
                                // is simply inner type with modifier
                                Type2::Typename { ident, .. } => {
                                    Ok(Self::Ident(ident.ident.into()))
                                },
                                _ => Err(format!(
                                    "other not implemented in paren: {:?}",
                                    tc.type1.type2
                                )),
                            }
                        },
                        // when multiple, it must be enum
                        _ => {
                            let mut variants = vec![];
                            for tc in &pt.type_choices {
                                match &tc.type1.type2 {
                                    Type2::IntValue { .. }
                                    | Type2::UintValue { .. }
                                    | Type2::FloatValue { .. } => {
                                        return Err(
                                            "numeric value not supported in paren enum".into()
                                        );
                                    },
                                    Type2::TextValue { value, .. } => {
                                        variants.push(EnumVariant::Literal(value.as_ref().into()));
                                    },
                                    Type2::Typename { ident, .. } => {
                                        variants.push(EnumVariant::Ident(ident.ident.into()));
                                    },
                                    Type2::ParenthesizedType { .. } => {
                                        return Err("nested paren type not implemented yet".into());
                                    },
                                    _ => {
                                        return Err(
                                            "other types not implemented yet in parens".into()
                                        );
                                    },
                                }
                            }
                            Ok(Self::Enum(variants))
                        },
                    },
                    // hashmap or inline map
                    Type2::Map { group, .. } => {
                        match &group.group_choices[..] {
                            [] => Err("group without choice is invalid in CDDL".into()),
                            [gc] => {
                                match &gc.group_entries[..] {
                                    [] => Err("empty inline struct is not implemented".into()),
                                    // single hashmap case {*text => text}
                                    [(GroupEntry::ValueMemberKey { ge, .. }, _)]
                                        if let Some(MemberKey::Type1 { .. }) = &ge.member_key
                                            && let [tc] = &ge.entry_type.type_choices[..]
                                            && let Type2::Typename { ident, .. } =
                                                &tc.type1.type2 =>
                                    {
                                        Ok(Self::HashMap(ident.ident.into()))
                                    },
                                    // or
                                    _ => {
                                        let mut fields = vec![];

                                        for (ge, _) in &gc.group_entries {
                                            match ge {
                                                GroupEntry::ValueMemberKey { ge, .. } => {
                                                    if let Some(MemberKey::Bareword {
                                                        ident, ..
                                                    }) = &ge.member_key
                                                    {
                                                        let skip_serializing = matches!(
                                                            ge.occur.as_ref().map(|o| o.occur),
                                                            Some(Occur::Optional { .. })
                                                        );
                                                        let name = ident.ident.into();
                                                        if let [tc] =
                                                            &ge.entry_type.type_choices[..]
                                                            && let Type2::Typename { ident, .. } =
                                                                &tc.type1.type2
                                                        {
                                                            fields.push(FieldVariant::Bareword(
                                                                skip_serializing,
                                                                name,
                                                                BarewordVariant::Ident(
                                                                    ident.ident.into(),
                                                                ),
                                                            ));
                                                        } else {
                                                            return Err("only direct typename is implemented in inline struct".into());
                                                        }
                                                    } else {
                                                        return Err("only bareword is implemented in inline struct".into());
                                                    }
                                                },
                                                GroupEntry::TypeGroupname { ge, .. } => {
                                                    fields.push(FieldVariant::Flatten(
                                                        ge.name.ident.into(),
                                                    ));
                                                },
                                                _ => {
                                                    return Err(
                                                        "inline group implemented in inline struct"
                                                            .into(),
                                                    );
                                                },
                                            }
                                        }

                                        Ok(Self::InlineStruct(fields))
                                    },
                                }
                            },
                            _ => Err(
                                "multiple group choices is not implemented for inline struct"
                                    .into(),
                            ),
                        }
                    },
                    Type2::Array { group, .. } => {
                        match &group.group_choices[..] {
                            [] => Err("group without group choices is invalid in CDDL".into()),
                            [gc] => {
                                if let [(ge, _)] = &gc.group_entries[..] {
                                    match ge {
                                        // `[ Foo ]`
                                        GroupEntry::TypeGroupname { ge, .. } => {
                                            Ok(Self::Vec(VecVariant::Ident(ge.name.ident.into())))
                                        },
                                        // `[foo: Bar]`, is this invalid?
                                        GroupEntry::ValueMemberKey { .. } => {
                                            Err("array members is not supported".into())
                                        },
                                        // `[ ( foo: Bar ) ]`, possible but not used in WebDriver
                                        GroupEntry::InlineGroup { group, .. } => {
                                            if let [gc] = &group.group_choices[..]
                                                && let [(GroupEntry::ValueMemberKey { ge, .. }, _)] =
                                                    &gc.group_entries[..]
                                            {
                                                let mut choices = vec![];
                                                for tc in &ge.entry_type.type_choices {
                                                    if let Type2::Typename { ident, .. } =
                                                        &tc.type1.type2
                                                    {
                                                        choices.push(ident.ident.into());
                                                    } else {
                                                        return Err(
                                                            "only typename is implemented".into()
                                                        );
                                                    }
                                                }
                                                Ok(Self::Vec(VecVariant::InlineChoice(choices)))
                                            } else {
                                                Err("array inline not implemented yet".into())
                                            }
                                        },
                                    }
                                } else {
                                    // `[foo: Bar, blah]`, rarely used in WebDriver
                                    Err("array without exactly many group entries is not implemented yet".into())
                                }
                            },
                            _ => Err("array group many not implemented yet".into()),
                        }
                    },

                    // below not supported
                    Type2::UTF8ByteString { .. }
                    | Type2::B16ByteString { .. }
                    | Type2::B64ByteString { .. } => {
                        Err("byte strings are not supported yet".into())
                    },
                    Type2::Unwrap { .. } => Err("unwrap is not supported".into()),
                    Type2::ChoiceFromInlineGroup { .. } => {
                        Err("choice in from inline group is not supported".into())
                    },
                    Type2::ChoiceFromGroup { .. } => {
                        Err("choice from group is not supported".into())
                    },
                    Type2::TaggedData { .. } => Err("tagged data is not supported".into()),
                    Type2::DataMajorType { .. } => Err("data major type is not supported".into()),
                    Type2::Any { .. } => Err("any is not allowed".into()),
                }
            },
            // directly multiple type case, must be enum
            _ => {
                let mut variants = vec![];

                for tc in &type_choices[..] {
                    match &tc.type1.type2 {
                        Type2::TextValue { value, .. } => {
                            variants.push(EnumVariant::Literal(value.as_ref().into()))
                        },
                        Type2::Typename { ident, .. } => {
                            variants.push(EnumVariant::Ident(ident.ident.into()));
                        },
                        Type2::ParenthesizedType { pt, .. }
                            if let Some(ident) = flatten_paren_as_identifier(pt) =>
                        {
                            variants.push(EnumVariant::Ident(ident.into()));
                        },
                        _ => {
                            return Err(format!(
                                "others not implemented in type choices: {:?}",
                                tc.type1.type2
                            ));
                        },
                    }
                }

                Ok(Self::Enum(variants))
            },
        }
    }
}

/// Some paren type is simply `( float .gt 0.0 )` and can be flatten to inner.
fn flatten_paren_as_identifier<'a>(pt: &'a Type) -> Option<&'a str> {
    if let [tc] = &pt.type_choices[..] {
        match &tc.type1.type2 {
            Type2::IntValue { .. } => Some("int"),
            Type2::UintValue { .. } => Some("uint"),
            Type2::FloatValue { .. } => Some("float"),
            Type2::Typename { ident, .. } => Some(ident.ident),
            _ => None,
        }
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub enum FieldVariant<'a> {
    // ordinary bareword fields
    Bareword(bool, Name<'a>, BarewordVariant<'a>),
    // group fields that should be flatten
    Flatten(Name<'a>),
    // inline groups
    Inline(InlineVariant<'a>),
}
