/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use style::Atom;
use style::stylesheets::FontFeatureValuesRule;
use style::stylesheets::font_feature_values_rule::{PairValues, SingleValue, VectorValues};

/// A key for looking up identifiers inside a [FontFeatureValueMap].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct HashKey {
    /// The name of the font that the feature name is defined for
    family_name: Atom,
    kind: AlternateKindRequiringResolution,
    /// The name of the feature.
    name: Atom,
}

/// Different kinds of values that identifiers inside `font-variant-alternates` can resolve to.
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

/// Enumerates the different variant kinds of [`font-variant-alternates`] which take a
/// one or more identifiers.
///
/// [`font-variant-alternates`]: https://drafts.csswg.org/css-fonts/#font-variant-alternates-prop
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum AlternateKindRequiringResolution {
    /// <https://drafts.csswg.org/css-fonts/#stylistic>
    Stylistic,
    /// <https://drafts.csswg.org/css-fonts/#styleset>
    Styleset,
    /// <https://drafts.csswg.org/css-fonts/#character-variant>
    CharacterVariant,
    /// <https://drafts.csswg.org/css-fonts/#swash>
    Swash,
    /// <https://drafts.csswg.org/css-fonts/#ornaments>
    Annotation,
    /// <https://drafts.csswg.org/css-fonts/#annotation>
    Ornaments,
}

impl FontFeatureValueMap {
    /// Looks up a single identifier in the map.
    pub(crate) fn lookup(
        &self,
        family_name: Atom,
        kind: AlternateKindRequiringResolution,
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

    /// Adds the values from a new `@font-feature-values` rule to the map.
    ///
    /// Conflicting entries are overwritten, so it is the callers responsibility to add the rules
    /// in ascending cascade order.
    pub(crate) fn add_rule(&mut self, rule: &FontFeatureValuesRule) {
        self.map.reserve(rule.family_names.len() * rule.len());
        for family_name in &rule.family_names {
            for swash in &rule.swash {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: AlternateKindRequiringResolution::Swash,
                    name: swash.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Single(swash.value.clone()));
            }

            for annotation in &rule.annotation {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: AlternateKindRequiringResolution::Annotation,
                    name: annotation.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Single(annotation.value.clone()));
            }

            for ornament in &rule.ornaments {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: AlternateKindRequiringResolution::Ornaments,
                    name: ornament.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Single(ornament.value.clone()));
            }

            for stylistic in &rule.stylistic {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: AlternateKindRequiringResolution::Stylistic,
                    name: stylistic.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Single(stylistic.value.clone()));
            }

            for styleset in &rule.styleset {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: AlternateKindRequiringResolution::Styleset,
                    name: styleset.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Vector(styleset.value.clone()));
            }

            for character_variant in &rule.character_variant {
                let key = HashKey {
                    family_name: family_name.name.clone(),
                    kind: AlternateKindRequiringResolution::CharacterVariant,
                    name: character_variant.name.clone(),
                };
                self.map
                    .insert(key, FontFeatureValue::Pair(character_variant.value.clone()));
            }
        }
    }
}
