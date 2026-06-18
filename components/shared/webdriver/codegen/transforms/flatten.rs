use std::borrow::Cow;

use crate::{
    ast::{Field, File, Name, Rule, Type, Visitor, walk_field, walk_rule, walk_ty},
    util::to_pascal_case,
};

const FLATTEN_RECURSION_LIMIT: usize = 255;

pub fn flatten_recursively(file: &mut File) {
    for _ in 0..FLATTEN_RECURSION_LIMIT {
        let mut visitor = FlattenVisitor::default();
        visitor.visit_file(file);
        if visitor.new_rules.is_empty() {
            return;
        }
        file.rules.extend(visitor.new_rules);
    }
    panic!("flatten recursion limit reached")
}
/// This visitor flattens all nested type into rules.
///
/// The overall naming rules (for inline only):
///
/// - `Array(_ty)` => `${Parent}Item`
/// - `Tuple([_ty1, _ty2])` => `${Parent}0`, `${Parent}1`
/// - `Map([Field::Keyed { key, .. }])` => `${Parent}${key}`
/// - `Map([Field::Inline(ty)])` => `${Parent}${ty.key}` or `${Parent}Inline`
#[derive(Default)]
pub struct FlattenVisitor<'a> {
    /// The outmost rule name.
    rule_name: Option<Name<'a>>,
    /// A stack of current path, used to derive name for inline type.
    path_stack: Vec<Cow<'a, str>>,
    /// The newly generated rule to merge
    pub new_rules: Vec<Rule<'a>>,
}

impl<'a> FlattenVisitor<'a> {
    fn derive_current_name(&self) -> Option<Name<'a>> {
        let path_iter = self.path_stack.iter().cloned();
        if self.path_stack.is_empty() {
            return None;
        }
        match self.rule_name.clone()? {
            Name::Global(raw) => {
                let joined = std::iter::once(raw)
                    .chain(path_iter)
                    .flat_map(|s| to_pascal_case(s.chars()).collect::<Vec<_>>())
                    .collect::<String>();
                Some(Name::Global(joined.into()))
            },
            Name::Prefixed { prefix, name, .. } => {
                let joined = std::iter::once(name)
                    .chain(path_iter)
                    .flat_map(|s| to_pascal_case(s.chars()).collect::<Vec<_>>())
                    .collect::<String>();
                let raw = format!("{prefix}.{joined}");
                Some(Name::Prefixed {
                    raw: raw.into(),
                    prefix: prefix.clone(),
                    name: joined.into(),
                })
            },
            Name::Primitive(_) => unreachable!(),
        }
    }
}

impl<'a> Visitor<'a> for FlattenVisitor<'a> {
    fn visit_rule(&mut self, rule: &mut Rule<'a>) {
        self.rule_name = Some(rule.name.clone());
        walk_rule(self, rule);
        self.rule_name = None;
    }

    fn visit_field(&mut self, field: &mut Field<'a>) {
        match field {
            Field::Keyed { key, .. } => {
                self.path_stack.push(key.clone());
                walk_field(self, field);
                self.path_stack.pop();
            },
            Field::Inline(inner) => {
                let key: Cow<'static, str> = derive_key_name(inner)
                    // fallback to `${Parent}Data` when it is hard to derive a name,
                    // like `(network.ContinueWithAuthCredentials // network.ContinueWithAuthNoCredentials)`
                    .unwrap_or_else(|| "inline".into())
                    .into();
                self.path_stack.push(key.clone());
                walk_field(self, field);
                // post process of flattening inline choice
                // we need to turn it into ordinary field.
                if let Field::Inline(Type::Named(name)) = field {
                    *field = Field::Keyed {
                        skip: false,
                        flatten: true,
                        key: key,
                        ty: Type::Named(name.clone()),
                    };
                }
                self.path_stack.pop();
            },
        }
    }

    fn visit_ty(&mut self, ty: &mut Type<'a>) {
        match ty {
            Type::Array(_) => {
                self.path_stack.push("Item".into());
                walk_ty(self, ty);
                self.path_stack.pop();
            },
            Type::Arrow(key, value) => {
                self.path_stack.push("Key".into());
                walk_ty(self, key);
                self.path_stack.pop();
                self.path_stack.push("Value".into());
                walk_ty(self, value);
                self.path_stack.pop();
            },
            Type::Tuple(items) => {
                for (idx, item) in items.iter_mut().enumerate() {
                    self.path_stack.push(idx.to_string().into());
                    walk_ty(self, item);
                    self.path_stack.pop();
                }
            },

            // Below, for inline map and choice, for which Rust cannot represent
            // inline, we extract their type to new rules, and replace the node with
            // Type::Named with the new derive name.
            Type::Map(_) | Type::Choices(_) => {
                match self.derive_current_name() {
                    Some(derived_name) => {
                        // push the inline rule to new rules
                        self.new_rules.push(Rule {
                            name: derived_name.clone(),
                            ty: ty.clone(),
                        });
                        // replace current ty node with named
                        // TODO: here inline named is not allowed, should change to field
                        *ty = Type::Named(derived_name);
                    },
                    None => walk_ty(self, ty),
                };
            },

            // TODO: literals not handled, should check in another step before if the literal is
            // a tag or a really field.
            _ => walk_ty(self, ty),
        }
    }
}

fn derive_key_name(ty: &Type) -> Option<String> {
    Some(match ty {
        Type::Named(name) => match name {
            Name::Primitive(_) => unreachable!(),
            Name::Global(cow) => cow.to_string(),
            Name::Prefixed { name, .. } => name.to_string(),
        },
        Type::Map(fields)
            if fields.len() == 1
                && let Some(Field::Keyed { key, .. }) = fields.first() =>
        {
            return Some(key.to_string());
        },

        _ => None?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_recursively() {
        // ```text
        // Foo = {
        //   bar: {
        //     (Blah // Bob)
        //   }
        // }
        // ```
        let mut input = File {
            rules: vec![Rule {
                name: "Foo".into(),
                ty: Type::Map(vec![Field::Keyed {
                    skip: false,
                    flatten: false,
                    key: "bar".into(),
                    ty: Type::Map(vec![Field::Inline(Type::Choices(vec![
                        Type::Named("Blah".into()),
                        Type::Named("Bob".into()),
                    ]))]),
                }]),
            }],
        };
        flatten_recursively(&mut input);
        assert_eq!(
            input.rules,
            vec![
                Rule {
                    name: "Foo".into(),
                    ty: Type::Map(vec![Field::Keyed {
                        skip: false,
                        flatten: false,
                        key: "bar".into(),
                        ty: Type::Named("FooBar".into())
                    }])
                },
                Rule {
                    name: "FooBar".into(),
                    ty: Type::Map(vec![Field::Keyed {
                        skip: false,
                        flatten: true,
                        key: "inline".into(),
                        ty: Type::Named("FooBarInline".into())
                    }])
                },
                Rule {
                    name: "FooBarInline".into(),
                    ty: Type::Choices(vec![Type::Named("Blah".into()), Type::Named("Bob".into()),])
                },
            ]
        );
    }
}
