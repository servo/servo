/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use style::Atom;
use style::stylesheets::FontFeatureValuesRule;
use style::stylesheets::font_feature_values_rule::{PairValues, SingleValue, VectorValues};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct HashKey {
    /// The name of the font that the feature name is defined for
    family_name: Atom,
    kind: FontFeatureValueKind,
    /// The name of the feature.
    name: Atom,
}

#[derive(Clone, Debug)]
pub(crate) enum FontFeatureValue {
    Single(SingleValue),
    Pair(PairValues),
    Vector(VectorValues),
}

/// Stores accumulated data of all active [`@font-feature-value`] rules.
///
/// [`@font-feature-value`]: https://drafts.csswg.org/css-fonts/#font-feature-values
#[derive(Debug, Default)]
pub(crate) struct FontFeatureValueMap {
    map: HashMap<HashKey, FontFeatureValue>,
}

/// Enumerates different kind of feature value blocks that accept a single value.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum FontFeatureValueKind {
    Swash,
    Annotation,
    Ornaments,
    Stylistic,
    Styleset,
    CharacterVariant,
}

impl FontFeatureValueMap {
    pub(crate) fn lookup(
        &self,
        family_name: Atom,
        kind: FontFeatureValueKind,
        name: Atom,
    ) -> Option<FontFeatureValue> {
        let key = HashKey {
            family_name,
            kind,
            name,
        };

        let value = self.map.get(&key).cloned();
        log::debug!(
            "@font-feature-values lookup for {:?} {:?} {:?} -> {value:?}",
            key.family_name,
            key.kind,
            key.name
        );
        value
    }

    pub(crate) fn add_rule(&mut self, rule: &FontFeatureValuesRule) {
        self.map.reserve(rule.family_names.len() * rule.len());
        for family_name in &rule.family_names {
            for swash in &rule.swash {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: FontFeatureValueKind::Swash,
                    name: swash.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Single(swash.value.clone()));
            }

            for annotation in &rule.annotation {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: FontFeatureValueKind::Annotation,
                    name: annotation.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Single(annotation.value.clone()));
            }

            for ornament in &rule.ornaments {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: FontFeatureValueKind::Ornaments,
                    name: ornament.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Single(ornament.value.clone()));
            }

            for stylistic in &rule.stylistic {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: FontFeatureValueKind::Stylistic,
                    name: stylistic.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Single(stylistic.value.clone()));
            }

            for styleset in &rule.styleset {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: FontFeatureValueKind::Styleset,
                    name: styleset.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Vector(styleset.value.clone()));
            }

            for character_variant in &rule.character_variant {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: FontFeatureValueKind::CharacterVariant,
                    name: character_variant.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Pair(character_variant.value.clone()));
            }
        }
    }
}
