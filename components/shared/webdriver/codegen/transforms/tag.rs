use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use indexmap::IndexMap;

use crate::ast::{Attr, Choice, Field, Name, Primitive, Rule, Type};

// TODO: doc: this is to handle the case of ..., (put an example here).
//
// All variant of a enum will together vote for tag field of this enum.
//
// Once that is elected, the tag field in the struct is hoisted into enum.
// And then variant without field degenerate to Unit and the corresponding struct is removed.
//
// All other variant should be marked as `#[serde(untagged)]`.
//
// Field With multiple literals are expanded into multiple variant.
//
// Rule:
// 1. At least two variant, and at least one literal.
// 2. text fallback
// 3. vote from nested enum passthrough and counts
//
// When there are multiple qualified candidate, the first is selected. because spec prefer to write tag field first.

fn run() {}

struct Vote {
    /// true if this vote is from a literal, false when from text
    pub(crate) literal: bool,
    /// true if this vote is from direct struct, false when from nested enum.
    pub(crate) direct: bool,
    /// The index of the voter field inside its struct.
    pub(crate) idx: usize,
}

/// Map from field name to its votes.
type VoteMap<'a> = IndexMap<Cow<'a, str>, HashMap<Name<'a>, Vote>>;

type Candidate<'a> = (Cow<'a, str>, HashMap<Name<'a>, Vote>);

/// Map from rule name to elected field.
type ElectedMap<'a> = HashMap<Name<'a>, Option<Candidate<'a>>>;

/// collect all mutation plan.
fn run_election<'a>(rules: &'a HashMap<Name<'a>, Rule<'a>>) -> ElectedMap<'a> {
    let mut map = HashMap::new();
    for rule in rules.values() {
        // only handle choices (enums)
        if let Type::Choices(choices) = &rule.ty {
            let mut candidates = VoteMap::new();
            collect_votes_for_choices(rules, choices, &mut candidates, true);
            let elected = candidates
                .into_iter()
                // 1. vote >= 2
                .filter(|(_k, v)| v.len() > 2)
                // 2. literal vote >= 1
                .filter(|(_k, v)| v.values().any(|v| v.literal))
                // 3. prefer first
                .next();
            map.insert(rule.name.clone(), elected);
        }
    }
    map
}

fn run_mutation<'a>(rule: &mut HashMap<Name<'a>, Rule<'a>>, elected_map: ElectedMap<'a>) {
    for (choice_name, elected) in elected_map {
        match elected {
            // if election fails,
            // we add untagged to all fields except for literals variant
            // fields themselves keep untouched
            None => {
                let Type::Choices(choices) = &mut rule.get_mut(&choice_name).unwrap().ty else {
                    unreachable!()
                };
                for choice in choices {
                    match choice.ty {
                        Type::Literals(_) => {},
                        _ => choice.attrs.push(Attr::Untagged),
                    }
                }
            },
            // if election succeed,
            // 1. for each direct voter, remove the elected field
            // 2. if that voter have not other field, degenerate and remove, record that in a set.
            // 3. if that voter have multiple
            // 4. for each variant in
            Some((elected_field_name, voters)) => {
                let mut split = HashMap::new();
                let mut degenerate = HashSet::new();
                // the struct part
                for (voter_name, vote) in voters.iter() {
                    // only on direct
                    if vote.direct {
                        let Type::Map(st) = &mut rule.get_mut(&voter_name).unwrap().ty else {
                            unreachable!("last step")
                        };
                        // remove the elected field
                        let voter_field = st.remove(vote.idx);
                        match voter_field {
                            Field::Keyed { ty, .. } => match ty {
                                Type::Literals(s) => {
                                    // multiple
                                    split.insert(voter_name.clone(), s);
                                },
                                _ => {},
                            },
                            Field::Inline(_) => {},
                        }
                        // degenerate: struct side
                        if st.len() == 0 {
                            rule.remove(&voter_name);
                            degenerate.insert(voter_name);
                        }
                    }
                }
                // the variants part
                let Type::Choices(choices) = &mut rule.get_mut(&choice_name).unwrap().ty else {
                    unreachable!()
                };
                for variant in choices {
                    match &mut variant.ty {
                        // if a field is originally literal, it should be unchanged
                        Type::Literals(_) => {},
                        Type::Array(_) | Type::Tuple(_) | Type::Arrow(_, _) | Type::Optional(_) => {
                            variant.attrs.push(Attr::Untagged);
                        },
                        Type::Named(name) => {
                            match voters.get(&name) {
                                // name is a nested enum
                                None => {
                                    variant.attrs.push(Attr::Untagged);
                                },
                                // name is a direct struct
                                Some(vote) => {
                                    // TODO: use the name provided by vote instead
                                    // may need to choices = choices.iter.flatmap.collect
                                },
                            }
                        },
                        _ => {},
                    }
                }
            },
        }
    }
}

fn collect_votes_for_choices<'a>(
    rules: &'a HashMap<Name<'a>, Rule<'a>>,
    choices: &'a Vec<Choice<'a>>,
    map: &mut VoteMap<'a>,
    direct: bool,
) {
    for variant in choices.iter() {
        match &variant.ty {
            Type::Map(_) | Type::Choices(_) => {
                unreachable!("inline map and choices are already flattened")
            },
            Type::Named(name) => {
                collect_votes_for_rule(rules, name, map, direct);
            },
            _ => {},
        }
    }
}

fn collect_votes_for_rule<'a>(
    rules: &'a HashMap<Name<'a>, Rule<'a>>,
    rule_name: &'a Name<'a>,
    vote_map: &mut VoteMap<'a>,
    direct: bool,
) {
    let rule = rules.get(rule_name).expect("missing definition");
    match &rule.ty {
        Type::Named(name) => collect_votes_for_rule(rules, name, vote_map, direct),
        Type::Map(fields) => {
            for (idx, field) in fields.iter().enumerate() {
                match field {
                    Field::Keyed { key, ty, .. } => match &ty {
                        Type::Literals(_) => {
                            vote_map.entry(key.clone()).or_default().insert(
                                rule_name.clone(),
                                Vote {
                                    literal: true,
                                    direct,
                                    idx,
                                },
                            );
                        },
                        Type::Named(name) => match name {
                            Name::Primitive(Primitive::Text) => {
                                vote_map.entry(key.clone()).or_default().insert(
                                    rule_name.clone(),
                                    Vote {
                                        literal: false,
                                        direct,
                                        idx,
                                    },
                                );
                            },
                            _ => {},
                        },
                        _ => {},
                    },
                    Field::Inline(_) => {
                        unreachable!("inline map is already flattened")
                    },
                }
            }
        },
        Type::Choices(choices) => {
            // here the choice is nested, so we make `direct` false.
            collect_votes_for_choices(rules, choices, vote_map, false);
        },
        _ => {},
    }
}
