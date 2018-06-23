/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A property declaration block.

#![deny(missing_docs)]

use context::QuirksMode;
use cssparser::{DeclarationListParser, parse_important, ParserInput, CowRcStr};
use cssparser::{Parser, AtRuleParser, DeclarationParser, Delimiter, ParseErrorKind};
use custom_properties::CustomPropertiesBuilder;
use error_reporting::{ParseErrorReporter, ContextualParseError};
use parser::ParserContext;
use properties::animated_properties::{AnimationValue, AnimationValueMap};
use shared_lock::Locked;
use smallbitvec::{self, SmallBitVec};
use smallvec::SmallVec;
use std::fmt::{self, Write};
use std::iter::{DoubleEndedIterator, Zip};
use std::slice::Iter;
use str::{CssString, CssStringBorrow, CssStringWriter};
use style_traits::{CssWriter, ParseError, ParsingMode, StyleParseErrorKind, ToCss};
use stylesheets::{CssRuleType, Origin, UrlExtraData};
use super::*;
use values::computed::Context;

/// The animation rules.
///
/// The first one is for Animation cascade level, and the second one is for
/// Transition cascade level.
pub struct AnimationRules(pub Option<Arc<Locked<PropertyDeclarationBlock>>>,
                          pub Option<Arc<Locked<PropertyDeclarationBlock>>>);

impl AnimationRules {
    /// Returns whether these animation rules represents an actual rule or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_none() && self.1.is_none()
    }
}

/// Whether a given declaration comes from CSS parsing, or from CSSOM.
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
pub enum DeclarationSource {
    /// The declaration was obtained from CSS parsing of sheets and such.
    Parsing,
    /// The declaration was obtained from CSSOM.
    CssOm,
}

/// A declaration [importance][importance].
///
/// [importance]: https://drafts.csswg.org/css-cascade/#importance
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
pub enum Importance {
    /// Indicates a declaration without `!important`.
    Normal,

    /// Indicates a declaration with `!important`.
    Important,
}

impl Importance {
    /// Return whether this is an important declaration.
    pub fn important(self) -> bool {
        match self {
            Importance::Normal => false,
            Importance::Important => true,
        }
    }
}

/// Overridden declarations are skipped.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone)]
pub struct PropertyDeclarationBlock {
    /// The group of declarations, along with their importance.
    ///
    /// Only deduplicated declarations appear here.
    declarations: Vec<PropertyDeclaration>,

    /// The "important" flag for each declaration in `declarations`.
    declarations_importance: SmallBitVec,

    longhands: LonghandIdSet,
}

/// Iterator over `(PropertyDeclaration, Importance)` pairs.
pub struct DeclarationImportanceIterator<'a> {
    iter: Zip<Iter<'a, PropertyDeclaration>, smallbitvec::Iter<'a>>,
}

impl<'a> DeclarationImportanceIterator<'a> {
    /// Constructor.
    pub fn new(declarations: &'a [PropertyDeclaration], important: &'a SmallBitVec) -> Self {
        DeclarationImportanceIterator {
            iter: declarations.iter().zip(important.iter()),
        }
    }
}

impl<'a> Iterator for DeclarationImportanceIterator<'a> {
    type Item = (&'a PropertyDeclaration, Importance);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(decl, important)|
            (decl, if important { Importance::Important } else { Importance::Normal }))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator for DeclarationImportanceIterator<'a> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(decl, important)|
            (decl, if important { Importance::Important } else { Importance::Normal }))
    }
}

/// Iterator over `PropertyDeclaration` for Importance::Normal.
pub struct NormalDeclarationIterator<'a>(DeclarationImportanceIterator<'a>);

impl<'a> NormalDeclarationIterator<'a> {
    /// Constructor.
    #[inline]
    pub fn new(declarations: &'a [PropertyDeclaration], important: &'a SmallBitVec) -> Self {
        NormalDeclarationIterator(
            DeclarationImportanceIterator::new(declarations, important)
        )
    }
}

impl<'a> Iterator for NormalDeclarationIterator<'a> {
    type Item = &'a PropertyDeclaration;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (decl, importance) = self.0.iter.next()?;
            if !importance {
                return Some(decl);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.iter.size_hint()
    }
}

/// Iterator for AnimationValue to be generated from PropertyDeclarationBlock.
pub struct AnimationValueIterator<'a, 'cx, 'cx_a:'cx> {
    iter: NormalDeclarationIterator<'a>,
    context: &'cx mut Context<'cx_a>,
    default_values: &'a ComputedValues,
    /// Custom properties in a keyframe if exists.
    extra_custom_properties: Option<&'a Arc<::custom_properties::CustomPropertiesMap>>,
}

impl<'a, 'cx, 'cx_a:'cx> AnimationValueIterator<'a, 'cx, 'cx_a> {
    fn new(
        declarations: &'a PropertyDeclarationBlock,
        context: &'cx mut Context<'cx_a>,
        default_values: &'a ComputedValues,
        extra_custom_properties: Option<&'a Arc<::custom_properties::CustomPropertiesMap>>,
    ) -> AnimationValueIterator<'a, 'cx, 'cx_a> {
        AnimationValueIterator {
            iter: declarations.normal_declaration_iter(),
            context,
            default_values,
            extra_custom_properties,
        }
    }
}

impl<'a, 'cx, 'cx_a:'cx> Iterator for AnimationValueIterator<'a, 'cx, 'cx_a> {
    type Item = AnimationValue;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let decl = self.iter.next()?;

            let animation = AnimationValue::from_declaration(
                decl,
                &mut self.context,
                self.extra_custom_properties,
                self.default_values,
            );

            if let Some(anim) = animation {
                return Some(anim);
            }
        }
    }
}

impl fmt::Debug for PropertyDeclarationBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.declarations.fmt(f)
    }
}

impl PropertyDeclarationBlock {
    /// Returns the number of declarations in the block.
    #[inline]
    pub fn len(&self) -> usize {
        self.declarations.len()
    }

    /// Create an empty block
    #[inline]
    pub fn new() -> Self {
        PropertyDeclarationBlock {
            declarations: Vec::new(),
            declarations_importance: SmallBitVec::new(),
            longhands: LonghandIdSet::new(),
        }
    }

    /// Create a block with a single declaration
    pub fn with_one(declaration: PropertyDeclaration, importance: Importance) -> Self {
        let mut longhands = LonghandIdSet::new();
        if let PropertyDeclarationId::Longhand(id) = declaration.id() {
            longhands.insert(id);
        }
        PropertyDeclarationBlock {
            declarations: vec![declaration],
            declarations_importance: SmallBitVec::from_elem(1, importance.important()),
            longhands,
        }
    }

    /// The declarations in this block
    #[inline]
    pub fn declarations(&self) -> &[PropertyDeclaration] {
        &self.declarations
    }

    /// The `important` flags for declarations in this block
    #[inline]
    pub fn declarations_importance(&self) -> &SmallBitVec {
        &self.declarations_importance
    }

    /// Iterate over `(PropertyDeclaration, Importance)` pairs
    #[inline]
    pub fn declaration_importance_iter(&self) -> DeclarationImportanceIterator {
        DeclarationImportanceIterator::new(&self.declarations, &self.declarations_importance)
    }

    /// Iterate over `PropertyDeclaration` for Importance::Normal
    #[inline]
    pub fn normal_declaration_iter(&self) -> NormalDeclarationIterator {
        NormalDeclarationIterator::new(&self.declarations, &self.declarations_importance)
    }

    /// Return an iterator of (AnimatableLonghand, AnimationValue).
    #[inline]
    pub fn to_animation_value_iter<'a, 'cx, 'cx_a:'cx>(
        &'a self,
        context: &'cx mut Context<'cx_a>,
        default_values: &'a ComputedValues,
        extra_custom_properties: Option<&'a Arc<::custom_properties::CustomPropertiesMap>>,
    ) -> AnimationValueIterator<'a, 'cx, 'cx_a> {
        AnimationValueIterator::new(self, context, default_values, extra_custom_properties)
    }

    /// Returns whether this block contains any declaration with `!important`.
    ///
    /// This is based on the `declarations_importance` bit-vector,
    /// which should be maintained whenever `declarations` is changed.
    #[inline]
    pub fn any_important(&self) -> bool {
        !self.declarations_importance.all_false()
    }

    /// Returns whether this block contains any declaration without `!important`.
    ///
    /// This is based on the `declarations_importance` bit-vector,
    /// which should be maintained whenever `declarations` is changed.
    #[inline]
    pub fn any_normal(&self) -> bool {
        !self.declarations_importance.all_true()
    }

    /// Returns whether this block contains a declaration of a given longhand.
    #[inline]
    pub fn contains(&self, id: LonghandId) -> bool {
        self.longhands.contains(id)
    }

    /// Returns whether this block contains any reset longhand.
    #[inline]
    pub fn contains_any_reset(&self) -> bool {
        self.longhands.contains_any_reset()
    }

    /// Get a declaration for a given property.
    ///
    /// NOTE: This is linear time in the case of custom properties or in the
    /// case the longhand is actually in the declaration block.
    #[inline]
    pub fn get(&self, property: PropertyDeclarationId) -> Option<(&PropertyDeclaration, Importance)> {
        if let PropertyDeclarationId::Longhand(id) = property {
            if !self.contains(id) {
                return None;
            }
        }

        self.declarations.iter().enumerate().find(|&(_, decl)| decl.id() == property).map(|(i, decl)| {
            let importance = if self.declarations_importance[i] {
                Importance::Important
            } else {
                Importance::Normal
            };
            (decl, importance)
        })
    }

    fn shorthand_to_css(
        &self,
        shorthand: ShorthandId,
        dest: &mut CssStringWriter,
    ) -> fmt::Result {
        // Step 1.2.1 of
        // https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-getpropertyvalue
        let mut list = SmallVec::<[&_; 10]>::new();
        let mut important_count = 0;

        // Step 1.2.2
        for longhand in shorthand.longhands() {
            // Step 1.2.2.1
            let declaration = self.get(PropertyDeclarationId::Longhand(longhand));

            // Step 1.2.2.2 & 1.2.2.3
            match declaration {
                Some((declaration, importance)) => {
                    list.push(declaration);
                    if importance.important() {
                        important_count += 1;
                    }
                },
                None => return Ok(()),
            }
        }

        // If there is one or more longhand with important, and one or more
        // without important, we don't serialize it as a shorthand.
        if important_count > 0 && important_count != list.len() {
            return Ok(());
        }

        // Step 1.2.3
        // We don't print !important when serializing individual properties,
        // so we treat this as a normal-importance property
        match shorthand.get_shorthand_appendable_value(list.iter().cloned()) {
            Some(appendable_value) => {
                append_declaration_value(dest, appendable_value)
            }
            None => return Ok(()),
        }
    }

    /// Find the value of the given property in this block and serialize it
    ///
    /// <https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-getpropertyvalue>
    pub fn property_value_to_css(&self, property: &PropertyId, dest: &mut CssStringWriter) -> fmt::Result {
        // Step 1.1: done when parsing a string to PropertyId

        // Step 1.2
        let longhand_or_custom = match property.as_shorthand() {
            Ok(shorthand) => return self.shorthand_to_css(shorthand, dest),
            Err(longhand_or_custom) => longhand_or_custom,
        };

        if let Some((value, _importance)) = self.get(longhand_or_custom) {
            // Step 2
            value.to_css(dest)
        } else {
            // Step 3
            Ok(())
        }
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-getpropertypriority>
    pub fn property_priority(&self, property: &PropertyId) -> Importance {
        // Step 1: done when parsing a string to PropertyId

        // Step 2
        match property.as_shorthand() {
            Ok(shorthand) => {
                // Step 2.1 & 2.2 & 2.3
                if shorthand.longhands().all(|l| {
                    self.get(PropertyDeclarationId::Longhand(l))
                        .map_or(false, |(_, importance)| importance.important())
                }) {
                    Importance::Important
                } else {
                    Importance::Normal
                }
            }
            Err(longhand_or_custom) => {
                // Step 3
                self.get(longhand_or_custom).map_or(Importance::Normal, |(_, importance)| importance)
            }
        }
    }

    /// Adds or overrides the declaration for a given property in this block.
    ///
    /// See the documentation of `push` to see what impact `source` has when the
    /// property is already there.
    pub fn extend(
        &mut self,
        mut drain: SourcePropertyDeclarationDrain,
        importance: Importance,
        source: DeclarationSource,
    ) -> bool {
        match source {
            DeclarationSource::Parsing => {
                let all_shorthand_len = match drain.all_shorthand {
                    AllShorthand::NotSet => 0,
                    AllShorthand::CSSWideKeyword(_) |
                    AllShorthand::WithVariables(_) => shorthands::ALL_SHORTHAND_MAX_LEN,
                };
                let push_calls_count =
                    drain.declarations.len() + all_shorthand_len;

                // With deduplication the actual length increase may be less than this.
                self.declarations.reserve(push_calls_count);
            }
            DeclarationSource::CssOm => {
                // Don't bother reserving space, since it's usually the case
                // that CSSOM just overrides properties and we don't need to use
                // more memory. See bug 1424346 for an example where this
                // matters.
                //
                // TODO: Would it be worth to call reserve() just if it's empty?
            }
        }

        let mut changed = false;
        for decl in &mut drain.declarations {
            changed |= self.push(
                decl,
                importance,
                source,
            );
        }
        match drain.all_shorthand {
            AllShorthand::NotSet => {}
            AllShorthand::CSSWideKeyword(keyword) => {
                for id in ShorthandId::All.longhands() {
                    let decl = PropertyDeclaration::CSSWideKeyword(
                        WideKeywordDeclaration { id, keyword },
                    );
                    changed |= self.push(
                        decl,
                        importance,
                        source,
                    );
                }
            }
            AllShorthand::WithVariables(unparsed) => {
                for id in ShorthandId::All.longhands() {
                    let decl = PropertyDeclaration::WithVariables(
                        VariableDeclaration { id, value: unparsed.clone() },
                    );
                    changed |= self.push(
                        decl,
                        importance,
                        source,
                    );
                }
            }
        }
        changed
    }

    /// Adds or overrides the declaration for a given property in this block.
    ///
    /// Depending on the value of `source`, this has a different behavior in the
    /// presence of another declaration with the same ID in the declaration
    /// block.
    ///
    ///   * For `DeclarationSource::Parsing`, this will not override a
    ///     declaration with more importance, and will ensure that, if inserted,
    ///     it's inserted at the end of the declaration block.
    ///
    ///   * For `DeclarationSource::CssOm`, this will override importance.
    ///
    /// Returns whether the declaration has changed.
    pub fn push(
        &mut self,
        declaration: PropertyDeclaration,
        importance: Importance,
        source: DeclarationSource,
    ) -> bool {
        let longhand_id = match declaration.id() {
            PropertyDeclarationId::Longhand(id) => Some(id),
            PropertyDeclarationId::Custom(..) => None,
        };

        let definitely_new = longhand_id.map_or(false, |id| {
            !self.longhands.contains(id)
        });

        if !definitely_new {
            let mut index_to_remove = None;
            for (i, slot) in self.declarations.iter_mut().enumerate() {
                if slot.id() != declaration.id() {
                    continue;
                }

                if matches!(source, DeclarationSource::Parsing) {
                    let important = self.declarations_importance[i];

                    // For declarations from parsing, non-important declarations
                    // shouldn't override existing important one.
                    if important && !importance.important() {
                        return false;
                    }

                    // As a compatibility hack, specially on Android,
                    // don't allow to override a prefixed webkit display
                    // value with an unprefixed version from parsing
                    // code.
                    //
                    // TODO(emilio): Unship.
                    if let PropertyDeclaration::Display(old_display) = *slot {
                        use properties::longhands::display::computed_value::T as display;

                        if let PropertyDeclaration::Display(new_display) = declaration {
                            if display::should_ignore_parsed_value(old_display, new_display) {
                                return false;
                            }
                        }
                    }
                }

                index_to_remove = Some(i);
                break;
            }

            if let Some(index) = index_to_remove {
                self.declarations.remove(index);
                self.declarations_importance.remove(index);
                self.declarations.push(declaration);
                self.declarations_importance.push(importance.important());
                return true;
            }
        }

        if let Some(id) = longhand_id {
            self.longhands.insert(id);
        }
        self.declarations.push(declaration);
        self.declarations_importance.push(importance.important());
        true
    }

    /// Returns the first declaration that would be removed by removing
    /// `property`.
    #[inline]
    pub fn first_declaration_to_remove(
        &self,
        property: &PropertyId,
    ) -> Option<usize> {
        if let Some(id) = property.longhand_id() {
            if !self.longhands.contains(id) {
                return None;
            }
        }

        self.declarations.iter().position(|declaration| {
            declaration.id().is_or_is_longhand_of(property)
        })
    }

    /// Removes a given declaration at a given index.
    #[inline]
    fn remove_declaration_at(&mut self, i: usize) {
        {
            let id = self.declarations[i].id();
            if let PropertyDeclarationId::Longhand(id) = id {
                self.longhands.remove(id);
            }
            self.declarations_importance.remove(i);
        }
        self.declarations.remove(i);
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-removeproperty>
    ///
    /// `first_declaration` needs to be the result of
    /// `first_declaration_to_remove`.
    #[inline]
    pub fn remove_property(
        &mut self,
        property: &PropertyId,
        first_declaration: usize,
    ) {
        debug_assert_eq!(
            Some(first_declaration),
            self.first_declaration_to_remove(property)
        );
        debug_assert!(self.declarations[first_declaration].id().is_or_is_longhand_of(property));

        self.remove_declaration_at(first_declaration);

        let shorthand = match property.as_shorthand() {
            Ok(s) => s,
            Err(_longhand_or_custom) => return,
        };

        let mut i = first_declaration;
        let mut len = self.len();
        while i < len {
            if !self.declarations[i].id().is_longhand_of(shorthand) {
                i += 1;
                continue;
            }

            self.remove_declaration_at(i);
            len -= 1;
        }
    }

    /// Take a declaration block known to contain a single property and serialize it.
    pub fn single_value_to_css(
        &self,
        property: &PropertyId,
        dest: &mut CssStringWriter,
        computed_values: Option<&ComputedValues>,
        custom_properties_block: Option<&PropertyDeclarationBlock>,
    ) -> fmt::Result {
        if let Ok(shorthand) = property.as_shorthand() {
            return self.shorthand_to_css(shorthand, dest);
        }

        // FIXME(emilio): Should this assert, or assert that the declaration is
        // the property we expect?
        let declaration = match self.declarations.get(0) {
            Some(d) => d,
            None => return Err(fmt::Error),
        };

        let custom_properties = if let Some(cv) = computed_values {
            // If there are extra custom properties for this declaration block,
            // factor them in too.
            if let Some(block) = custom_properties_block {
                // FIXME(emilio): This is not super-efficient here, and all this
                // feels like a hack anyway...
                block.cascade_custom_properties(cv.custom_properties())
            } else {
                cv.custom_properties().cloned()
            }
        } else {
            None
        };

        match (declaration, computed_values) {
            // If we have a longhand declaration with variables, those variables
            // will be stored as unparsed values.
            //
            // As a temporary measure to produce sensible results in Gecko's
            // getKeyframes() implementation for CSS animations, if
            // |computed_values| is supplied, we use it to expand such variable
            // declarations. This will be fixed properly in Gecko bug 1391537.
            (
                &PropertyDeclaration::WithVariables(ref declaration),
                Some(ref _computed_values),
            ) => {
                declaration.value.substitute_variables(
                    declaration.id,
                    custom_properties.as_ref(),
                    QuirksMode::NoQuirks,
                ).to_css(dest)
            },
            (ref d, _) => d.to_css(dest),
        }
    }

    /// Convert AnimationValueMap to PropertyDeclarationBlock.
    pub fn from_animation_value_map(animation_value_map: &AnimationValueMap) -> Self {
        let len = animation_value_map.len();
        let mut declarations = Vec::with_capacity(len);
        let mut longhands = LonghandIdSet::new();

        for (property, animation_value) in animation_value_map.iter() {
          longhands.insert(*property);
          declarations.push(animation_value.uncompute());
        }

        PropertyDeclarationBlock {
            declarations,
            longhands,
            declarations_importance: SmallBitVec::from_elem(len, false),
        }
    }

    /// Returns true if the declaration block has a CSSWideKeyword for the given
    /// property.
    pub fn has_css_wide_keyword(&self, property: &PropertyId) -> bool {
        if let Some(id) = property.longhand_id() {
            if !self.longhands.contains(id) {
                return false
            }
        }
        self.declarations.iter().any(|decl|
            decl.id().is_or_is_longhand_of(property) &&
            decl.get_css_wide_keyword().is_some()
        )
    }

    /// Returns a custom properties map which is the result of cascading custom
    /// properties in this declaration block along with context's custom
    /// properties.
    pub fn cascade_custom_properties_with_context(
        &self,
        context: &Context,
    ) -> Option<Arc<::custom_properties::CustomPropertiesMap>> {
        self.cascade_custom_properties(context.style().custom_properties())
    }

    /// Returns a custom properties map which is the result of cascading custom
    /// properties in this declaration block along with the given custom
    /// properties.
    pub fn cascade_custom_properties(
        &self,
        inherited_custom_properties: Option<&Arc<::custom_properties::CustomPropertiesMap>>,
    ) -> Option<Arc<::custom_properties::CustomPropertiesMap>> {
        let mut builder = CustomPropertiesBuilder::new(inherited_custom_properties);

        for declaration in self.normal_declaration_iter() {
            if let PropertyDeclaration::Custom(ref declaration) = *declaration {
                builder.cascade(&declaration.name, declaration.value.borrow());
            }
        }

        builder.build()
    }

    /// Like the method on ToCss, but without the type parameter to avoid
    /// accidentally monomorphizing this large function multiple times for
    /// different writers.
    ///
    /// https://drafts.csswg.org/cssom/#serialize-a-css-declaration-block
    pub fn to_css(&self, dest: &mut CssStringWriter) -> fmt::Result {
        use std::iter::Cloned;
        use std::slice;

        let mut is_first_serialization = true; // trailing serializations should have a prepended space

        // Step 1 -> dest = result list

        // Step 2
        //
        // NOTE(emilio): We reuse this set for both longhands and shorthands
        // with subtly different meaning. For longhands, only longhands that
        // have actually been serialized (either by themselves, or as part of a
        // shorthand) appear here. For shorthands, all the shorthands that we've
        // attempted to serialize appear here.
        let mut already_serialized = NonCustomPropertyIdSet::new();

        // Step 3
        for (declaration, importance) in self.declaration_importance_iter() {
            // Step 3.1
            let property = declaration.id();
            let longhand_id = match property {
                PropertyDeclarationId::Longhand(id) => id,
                PropertyDeclarationId::Custom(..) => {
                    // Given the invariants that there are no duplicate
                    // properties in a declaration block, and that custom
                    // properties can't be part of a shorthand, we can just care
                    // about them here.
                    append_serialization::<Cloned<slice::Iter<_>>, _>(
                        dest,
                        &property,
                        AppendableValue::Declaration(declaration),
                        importance,
                        &mut is_first_serialization,
                    )?;
                    continue;
                }
            };

            // Step 3.2
            if already_serialized.contains(longhand_id.into()) {
                continue;
            }

            // Step 3.3
            // Step 3.3.1 is done by checking already_serialized while
            // iterating below.

            // Step 3.3.2
            for shorthand in longhand_id.shorthands() {
                // We already attempted to serialize this shorthand before.
                if already_serialized.contains(shorthand.into()) {
                    continue;
                }
                already_serialized.insert(shorthand.into());

                // Substep 2 & 3
                let mut current_longhands = SmallVec::<[_; 10]>::new();
                let mut important_count = 0;
                let mut found_system = None;

                let is_system_font =
                    shorthand == ShorthandId::Font &&
                    self.declarations.iter().any(|l| {
                        match l.id() {
                            PropertyDeclarationId::Longhand(id) => {
                                if already_serialized.contains(id.into()) {
                                    return false;
                                }

                                l.get_system().is_some()
                            }
                            PropertyDeclarationId::Custom(..) => {
                                debug_assert!(l.get_system().is_none());
                                false
                            }
                        }
                    });

                if is_system_font {
                    for (longhand, importance) in self.declaration_importance_iter() {
                        if longhand.get_system().is_some() || longhand.is_default_line_height() {
                            current_longhands.push(longhand);
                            if found_system.is_none() {
                               found_system = longhand.get_system();
                            }
                            if importance.important() {
                                important_count += 1;
                            }
                        }
                    }
                } else {
                    let mut contains_all_longhands = true;
                    for longhand in shorthand.longhands() {
                        match self.get(PropertyDeclarationId::Longhand(longhand)) {
                            Some((declaration, importance)) => {
                                current_longhands.push(declaration);
                                if importance.important() {
                                    important_count += 1;
                                }
                            }
                            None => {
                                contains_all_longhands = false;
                                break;
                            }
                        }
                    }

                    // Substep 1:
                    if !contains_all_longhands {
                        continue;
                    }
                }

                // Substep 4
                let is_important = important_count > 0;
                if is_important && important_count != current_longhands.len() {
                    continue;
                }
                let importance = if is_important {
                    Importance::Important
                } else {
                    Importance::Normal
                };

                // Substep 5 - Let value be the result of invoking serialize
                // a CSS value of current longhands.
                let appendable_value =
                    match shorthand.get_shorthand_appendable_value(current_longhands.iter().cloned()) {
                        None => continue,
                        Some(appendable_value) => appendable_value,
                    };

                // We avoid re-serializing if we're already an
                // AppendableValue::Css.
                let mut v = CssString::new();
                let value = match (appendable_value, found_system) {
                    (AppendableValue::Css { css, with_variables }, _) => {
                        debug_assert!(!css.is_empty());
                        AppendableValue::Css {
                            css: css,
                            with_variables: with_variables,
                        }
                    }
                    #[cfg(feature = "gecko")]
                    (_, Some(sys)) => {
                        sys.to_css(&mut CssWriter::new(&mut v))?;
                        AppendableValue::Css {
                            css: CssStringBorrow::from(&v),
                            with_variables: false,
                        }
                    }
                    (other, _) => {
                        append_declaration_value(&mut v, other)?;

                        // Substep 6
                        if v.is_empty() {
                            continue;
                        }

                        AppendableValue::Css {
                            css: CssStringBorrow::from(&v),
                            with_variables: false,
                        }
                    }
                };

                // Substeps 7 and 8
                append_serialization::<Cloned<slice::Iter<_>>, _>(
                    dest,
                    &shorthand,
                    value,
                    importance,
                    &mut is_first_serialization,
                )?;

                for current_longhand in &current_longhands {
                    let longhand_id = match current_longhand.id() {
                        PropertyDeclarationId::Longhand(id) => id,
                        PropertyDeclarationId::Custom(..) => unreachable!(),
                    };

                    // Substep 9
                    already_serialized.insert(longhand_id.into());
                }

                // FIXME(https://github.com/w3c/csswg-drafts/issues/1774)
                // The specification does not include an instruction to abort
                // the shorthand loop at this point, but doing so both matches
                // Gecko and makes sense since shorthands are checked in
                // preferred order.
                break;
            }

            // Step 3.3.4
            if already_serialized.contains(longhand_id.into()) {
                continue;
            }

            // Steps 3.3.5, 3.3.6 & 3.3.7
            // Need to specify an iterator type here even though it’s unused to work around
            // "error: unable to infer enough type information about `_`;
            //  type annotations or generic parameter binding required [E0282]"
            // Use the same type as earlier call to reuse generated code.
            append_serialization::<Cloned<slice::Iter<_>>, _>(
                dest,
                &property,
                AppendableValue::Declaration(declaration),
                importance,
                &mut is_first_serialization,
            )?;

            // Step 3.3.8
            already_serialized.insert(longhand_id.into());
        }

        // Step 4
        Ok(())
    }
}

/// A convenient enum to represent different kinds of stuff that can represent a
/// _value_ in the serialization of a property declaration.
pub enum AppendableValue<'a, I>
    where I: Iterator<Item=&'a PropertyDeclaration>,
{
    /// A given declaration, of which we'll serialize just the value.
    Declaration(&'a PropertyDeclaration),
    /// A set of declarations for a given shorthand.
    ///
    /// FIXME: This needs more docs, where are the shorthands expanded? We print
    /// the property name before-hand, don't we?
    DeclarationsForShorthand(ShorthandId, I),
    /// A raw CSS string, coming for example from a property with CSS variables,
    /// or when storing a serialized shorthand value before appending directly.
    Css {
        /// The raw CSS string.
        css: CssStringBorrow<'a>,
        /// Whether the original serialization contained variables or not.
        with_variables: bool,
    }
}

/// Potentially appends whitespace after the first (property: value;) pair.
fn handle_first_serialization<W>(
    dest: &mut W,
    is_first_serialization: &mut bool,
) -> fmt::Result
where
    W: Write,
{
    if !*is_first_serialization {
        dest.write_str(" ")
    } else {
        *is_first_serialization = false;
        Ok(())
    }
}

/// Append a given kind of appendable value to a serialization.
pub fn append_declaration_value<'a, I>(
    dest: &mut CssStringWriter,
    appendable_value: AppendableValue<'a, I>,
) -> fmt::Result
where
    I: Iterator<Item=&'a PropertyDeclaration>,
{
    match appendable_value {
        AppendableValue::Css { css, .. } => {
            css.append_to(dest)
        },
        AppendableValue::Declaration(decl) => {
            decl.to_css(dest)
        },
        AppendableValue::DeclarationsForShorthand(shorthand, decls) => {
            shorthand.longhands_to_css(decls, &mut CssWriter::new(dest))
        }
    }
}

/// Append a given property and value pair to a serialization.
pub fn append_serialization<'a, I, N>(
    dest: &mut CssStringWriter,
    property_name: &N,
    appendable_value: AppendableValue<'a, I>,
    importance: Importance,
    is_first_serialization: &mut bool
) -> fmt::Result
where
    I: Iterator<Item=&'a PropertyDeclaration>,
    N: ToCss,
{
    handle_first_serialization(dest, is_first_serialization)?;

    property_name.to_css(&mut CssWriter::new(dest))?;
    dest.write_char(':')?;

    // for normal parsed values, add a space between key: and value
    match appendable_value {
        AppendableValue::Declaration(decl) => {
            if !decl.value_is_unparsed() {
                // For normal parsed values, add a space between key: and value.
                dest.write_str(" ")?
            }
        },
        AppendableValue::Css { with_variables, .. } => {
            if !with_variables {
                dest.write_str(" ")?
            }
        }
        // Currently append_serialization is only called with a Css or
        // a Declaration AppendableValue.
        AppendableValue::DeclarationsForShorthand(..) => unreachable!(),
    }

    append_declaration_value(dest, appendable_value)?;

    if importance.important() {
        dest.write_str(" !important")?;
    }

    dest.write_char(';')
}

/// A helper to parse the style attribute of an element, in order for this to be
/// shared between Servo and Gecko.
///
/// Inline because we call this cross-crate.
#[inline]
pub fn parse_style_attribute(
    input: &str,
    url_data: &UrlExtraData,
    error_reporter: Option<&ParseErrorReporter>,
    quirks_mode: QuirksMode,
) -> PropertyDeclarationBlock {
    let context = ParserContext::new(
        Origin::Author,
        url_data,
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        quirks_mode,
        error_reporter,
    );

    let mut input = ParserInput::new(input);
    parse_property_declaration_list(&context, &mut Parser::new(&mut input))
}

/// Parse a given property declaration. Can result in multiple
/// `PropertyDeclaration`s when expanding a shorthand, for example.
///
/// This does not attempt to parse !important at all.
#[inline]
pub fn parse_one_declaration_into(
    declarations: &mut SourcePropertyDeclaration,
    id: PropertyId,
    input: &str,
    url_data: &UrlExtraData,
    error_reporter: Option<&ParseErrorReporter>,
    parsing_mode: ParsingMode,
    quirks_mode: QuirksMode
) -> Result<(), ()> {
    let context = ParserContext::new(
        Origin::Author,
        url_data,
        Some(CssRuleType::Style),
        parsing_mode,
        quirks_mode,
        error_reporter,
    );

    let mut input = ParserInput::new(input);
    let mut parser = Parser::new(&mut input);
    let start_position = parser.position();
    parser.parse_entirely(|parser| {
        PropertyDeclaration::parse_into(declarations, id, &context, parser)
    }).map_err(|err| {
        let location = err.location;
        let error = ContextualParseError::UnsupportedPropertyDeclaration(
            parser.slice_from(start_position),
            err,
        );
        context.log_css_error(location, error);
    })
}

/// A struct to parse property declarations.
struct PropertyDeclarationParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
    declarations: &'a mut SourcePropertyDeclaration,
}


/// Default methods reject all at rules.
impl<'a, 'b, 'i> AtRuleParser<'i> for PropertyDeclarationParser<'a, 'b> {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = Importance;
    type Error = StyleParseErrorKind<'i>;
}

/// Based on NonMozillaVendorIdentifier from Gecko's CSS parser.
fn is_non_mozilla_vendor_identifier(name: &str) -> bool {
    (name.starts_with("-") && !name.starts_with("-moz-")) ||
        name.starts_with("_")
}

impl<'a, 'b, 'i> DeclarationParser<'i> for PropertyDeclarationParser<'a, 'b> {
    type Declaration = Importance;
    type Error = StyleParseErrorKind<'i>;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Importance, ParseError<'i>> {
        let id = match PropertyId::parse(&name, self.context) {
            Ok(id) => id,
            Err(..) => {
                return Err(input.new_custom_error(if is_non_mozilla_vendor_identifier(&name) {
                    StyleParseErrorKind::UnknownVendorProperty
                } else {
                    StyleParseErrorKind::UnknownProperty(name)
                }));
            }
        };
        input.parse_until_before(Delimiter::Bang, |input| {
            PropertyDeclaration::parse_into(self.declarations, id, self.context, input)
        })?;
        let importance = match input.try(parse_important) {
            Ok(()) => Importance::Important,
            Err(_) => Importance::Normal,
        };
        // In case there is still unparsed text in the declaration, we should roll back.
        input.expect_exhausted()?;
        Ok(importance)
    }
}


/// Parse a list of property declarations and return a property declaration
/// block.
pub fn parse_property_declaration_list(
    context: &ParserContext,
    input: &mut Parser,
) -> PropertyDeclarationBlock {
    let mut declarations = SourcePropertyDeclaration::new();
    let mut block = PropertyDeclarationBlock::new();
    let parser = PropertyDeclarationParser {
        context: context,
        declarations: &mut declarations,
    };
    let mut iter = DeclarationListParser::new(input, parser);
    while let Some(declaration) = iter.next() {
        match declaration {
            Ok(importance) => {
                block.extend(
                    iter.parser.declarations.drain(),
                    importance,
                    DeclarationSource::Parsing,
                );
            }
            Err((error, slice)) => {
                iter.parser.declarations.clear();

                // If the unrecognized property looks like a vendor-specific property,
                // silently ignore it instead of polluting the error output.
                if let ParseErrorKind::Custom(StyleParseErrorKind::UnknownVendorProperty) = error.kind {
                    continue;
                }

                let location = error.location;
                let error = ContextualParseError::UnsupportedPropertyDeclaration(slice, error);
                context.log_css_error(location, error);
            }
        }
    }
    block
}
