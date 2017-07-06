/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A property declaration block.

#![deny(missing_docs)]

use context::QuirksMode;
use cssparser::{DeclarationListParser, parse_important, ParserInput, CompactCowStr};
use cssparser::{Parser, AtRuleParser, DeclarationParser, Delimiter};
use error_reporting::{ParseErrorReporter, ContextualParseError};
use parser::{ParserContext, log_css_error};
use properties::animated_properties::AnimationValue;
use selectors::parser::SelectorParseError;
use shared_lock::Locked;
use smallvec::SmallVec;
use std::fmt;
use std::slice::Iter;
use style_traits::{PARSING_MODE_DEFAULT, ToCss, ParseError, ParsingMode, StyleParseError};
use stylesheets::{CssRuleType, Origin, UrlExtraData};
use stylesheets::{MallocSizeOf, MallocSizeOfFn};
use super::*;
use values::computed::Context;
#[cfg(feature = "gecko")] use properties::animated_properties::AnimationValueMap;

/// The animation rules.
///
/// The first one is for Animation cascade level, and the second one is for
/// Transition cascade level.
pub struct AnimationRules<'a>(pub Option<&'a Arc<Locked<PropertyDeclarationBlock>>>,
                              pub Option<&'a Arc<Locked<PropertyDeclarationBlock>>>);

impl<'a> AnimationRules<'a> {
    /// Returns whether these animation rules represents an actual rule or not.
    pub fn is_empty(&self) -> bool {
        self.0.is_none() && self.1.is_none()
    }
}

/// A declaration [importance][importance].
///
/// [importance]: https://drafts.csswg.org/css-cascade/#importance
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Importance {
    /// Indicates a declaration without `!important`.
    Normal,

    /// Indicates a declaration with `!important`.
    Important,
}

impl MallocSizeOf for Importance {
    fn malloc_size_of_children(&self, _malloc_size_of: MallocSizeOfFn) -> usize {
        0
    }
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
#[derive(Clone)]
pub struct PropertyDeclarationBlock {
    /// The group of declarations, along with their importance.
    ///
    /// Only deduplicated declarations appear here.
    declarations: Vec<(PropertyDeclaration, Importance)>,

    /// The number of entries in `self.declaration` with `Importance::Important`
    important_count: usize,

    longhands: LonghandIdSet,
}

impl MallocSizeOf for PropertyDeclarationBlock {
    fn malloc_size_of_children(&self, malloc_size_of: MallocSizeOfFn) -> usize {
        self.declarations.malloc_size_of_children(malloc_size_of)
    }
}

/// Iterator for PropertyDeclaration to be generated from PropertyDeclarationBlock.
#[derive(Clone)]
pub struct PropertyDeclarationIterator<'a> {
    iter: Iter<'a, (PropertyDeclaration, Importance)>,
}

impl<'a> Iterator for PropertyDeclarationIterator<'a> {
    type Item = &'a PropertyDeclaration;
    #[inline]
    fn next(&mut self) -> Option<&'a PropertyDeclaration> {
        // we use this function because a closure won't be `Clone`
        fn get_declaration(dec: &(PropertyDeclaration, Importance))
            -> &PropertyDeclaration {
            &dec.0
        }
        self.iter.next().map(get_declaration as fn(_) -> _)
    }
}

/// Iterator for AnimationValue to be generated from PropertyDeclarationBlock.
pub struct AnimationValueIterator<'a, 'cx, 'cx_a:'cx> {
    iter: Iter<'a, (PropertyDeclaration, Importance)>,
    context: &'cx mut Context<'cx_a>,
    default_values: &'a Arc<ComputedValues>,
    reporter: &'a ParseErrorReporter,
}

impl<'a, 'cx, 'cx_a:'cx> AnimationValueIterator<'a, 'cx, 'cx_a> {
    fn new(declarations: &'a PropertyDeclarationBlock,
           context: &'cx mut Context<'cx_a>,
           default_values: &'a Arc<ComputedValues>,
           reporter: &'a ParseErrorReporter) -> AnimationValueIterator<'a, 'cx, 'cx_a> {
        AnimationValueIterator {
            iter: declarations.declarations().iter(),
            context: context,
            default_values: default_values,
            reporter: reporter
        }
    }
}

impl<'a, 'cx, 'cx_a:'cx> Iterator for AnimationValueIterator<'a, 'cx, 'cx_a> {
    type Item = (AnimatableLonghand, AnimationValue);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        use properties::Importance;

        loop {
            let next = self.iter.next();
            match next {
                Some(&(ref decl, importance)) => {
                    if importance == Importance::Normal {
                        let property = AnimatableLonghand::from_declaration(decl);
                        let animation = AnimationValue::from_declaration(decl, &mut self.context,
                                                                         self.default_values,
                                                                         self.reporter);
                        debug_assert!(property.is_none() == animation.is_none(),
                                      "The failure condition of AnimatableLonghand::from_declaration \
                                       and AnimationValue::from_declaration should be the same");
                        // Skip the property if either ::from_declaration fails.
                        match (property, animation) {
                            (Some(p), Some(a)) => return Some((p, a)),
                            (_, _) => {},
                        }
                    }
                },
                None => return None,
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
    pub fn len(&self) -> usize {
        self.declarations.len()
    }

    /// Create an empty block
    pub fn new() -> Self {
        PropertyDeclarationBlock {
            declarations: Vec::new(),
            important_count: 0,
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
            declarations: vec![(declaration, importance)],
            important_count: if importance.important() { 1 } else { 0 },
            longhands: longhands,
        }
    }

    /// The declarations in this block
    pub fn declarations(&self) -> &[(PropertyDeclaration, Importance)] {
        &self.declarations
    }

    /// Iterate over only PropertyDeclaration.
    pub fn declarations_iter(&self) -> PropertyDeclarationIterator {
        PropertyDeclarationIterator {
            iter: self.declarations.iter(),
        }
    }

    /// Return an iterator of (AnimatableLonghand, AnimationValue).
    pub fn to_animation_value_iter<'a, 'cx, 'cx_a:'cx>(&'a self,
                                                       context: &'cx mut Context<'cx_a>,
                                                       default_values: &'a Arc<ComputedValues>,
                                                       reporter: &'a ParseErrorReporter)
                                                       -> AnimationValueIterator<'a, 'cx, 'cx_a> {
        AnimationValueIterator::new(self, context, default_values, reporter)
    }

    /// Returns whether this block contains any declaration with `!important`.
    ///
    /// This is based on the `important_count` counter,
    /// which should be maintained whenever `declarations` is changed.
    // FIXME: make fields private and maintain it here in methods?
    pub fn any_important(&self) -> bool {
        self.important_count > 0
    }

    /// Returns whether this block contains any declaration without `!important`.
    ///
    /// This is based on the `important_count` counter,
    /// which should be maintained whenever `declarations` is changed.
    // FIXME: make fields private and maintain it here in methods?
    pub fn any_normal(&self) -> bool {
        self.declarations.len() > self.important_count
    }

    /// Get a declaration for a given property.
    ///
    /// NOTE: This is linear time.
    pub fn get(&self, property: PropertyDeclarationId) -> Option< &(PropertyDeclaration, Importance)> {
        self.declarations.iter().find(|&&(ref decl, _)| decl.id() == property)
    }

    /// Find the value of the given property in this block and serialize it
    ///
    /// https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    pub fn property_value_to_css<W>(&self, property: &PropertyId, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        // Step 1.1: done when parsing a string to PropertyId

        // Step 1.2
        match property.as_shorthand() {
            Ok(shorthand) => {
                // Step 1.2.1
                let mut list = Vec::new();
                let mut important_count = 0;

                // Step 1.2.2
                for &longhand in shorthand.longhands() {
                    // Step 1.2.2.1
                    let declaration = self.get(PropertyDeclarationId::Longhand(longhand));

                    // Step 1.2.2.2 & 1.2.2.3
                    match declaration {
                        Some(&(ref declaration, importance)) => {
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
                match shorthand.get_shorthand_appendable_value(list) {
                    Some(appendable_value) =>
                        append_declaration_value(dest, appendable_value),
                    None => return Ok(()),
                }
            }
            Err(longhand_or_custom) => {
                if let Some(&(ref value, _importance)) = self.get(longhand_or_custom) {
                    // Step 2
                    value.to_css(dest)
                } else {
                    // Step 3
                    Ok(())
                }
            }
        }
    }

    /// https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertypriority
    pub fn property_priority(&self, property: &PropertyId) -> Importance {
        // Step 1: done when parsing a string to PropertyId

        // Step 2
        match property.as_shorthand() {
            Ok(shorthand) => {
                // Step 2.1 & 2.2 & 2.3
                if shorthand.longhands().iter().all(|&l| {
                    self.get(PropertyDeclarationId::Longhand(l))
                        .map_or(false, |&(_, importance)| importance.important())
                }) {
                    Importance::Important
                } else {
                    Importance::Normal
                }
            }
            Err(longhand_or_custom) => {
                // Step 3
                self.get(longhand_or_custom).map_or(Importance::Normal, |&(_, importance)| importance)
            }
        }
    }

    /// Adds or overrides the declaration for a given property in this block,
    /// **except** if an existing declaration for the same property is more
    /// important.
    ///
    /// Always ensures that the property declaration is at the end.
    pub fn extend(&mut self, drain: SourcePropertyDeclarationDrain, importance: Importance) {
        self.extend_common(drain, importance, false);
    }

    /// Adds or overrides the declaration for a given property in this block,
    /// **even** if an existing declaration for the same property is more
    /// important, and reuses the same position in the block.
    ///
    /// Returns whether anything changed.
    pub fn extend_reset(&mut self, drain: SourcePropertyDeclarationDrain,
                        importance: Importance) -> bool {
        self.extend_common(drain, importance, true)
    }

    fn extend_common(
        &mut self,
        mut drain: SourcePropertyDeclarationDrain,
        importance: Importance,
        overwrite_more_important_and_reuse_slot: bool,
    ) -> bool {
        let all_shorthand_len = match drain.all_shorthand {
            AllShorthand::NotSet => 0,
            AllShorthand::CSSWideKeyword(_) |
            AllShorthand::WithVariables(_) => ShorthandId::All.longhands().len()
        };
        let push_calls_count = drain.declarations.len() + all_shorthand_len;

        // With deduplication the actual length increase may be less than this.
        self.declarations.reserve(push_calls_count);

        let mut changed = false;
        for decl in &mut drain.declarations {
            changed |= self.push_common(
                decl,
                importance,
                overwrite_more_important_and_reuse_slot,
            );
        }
        match drain.all_shorthand {
            AllShorthand::NotSet => {}
            AllShorthand::CSSWideKeyword(keyword) => {
                for &id in ShorthandId::All.longhands() {
                    let decl = PropertyDeclaration::CSSWideKeyword(id, keyword);
                    changed |= self.push_common(
                        decl,
                        importance,
                        overwrite_more_important_and_reuse_slot,
                    );
                }
            }
            AllShorthand::WithVariables(unparsed) => {
                for &id in ShorthandId::All.longhands() {
                    let decl = PropertyDeclaration::WithVariables(id, unparsed.clone());
                    changed |= self.push_common(
                        decl,
                        importance,
                        overwrite_more_important_and_reuse_slot,
                    );
                }
            }
        }
        changed
    }

    /// Adds or overrides the declaration for a given property in this block,
    /// **except** if an existing declaration for the same property is more
    /// important.
    ///
    /// Ensures that, if inserted, it's inserted at the end of the declaration
    /// block.
    pub fn push(&mut self, declaration: PropertyDeclaration, importance: Importance) {
        self.push_common(declaration, importance, false);
    }

    fn push_common(
        &mut self,
        declaration: PropertyDeclaration,
        importance: Importance,
        overwrite_more_important_and_reuse_slot: bool
    ) -> bool {
        let definitely_new = if let PropertyDeclarationId::Longhand(id) = declaration.id() {
            !self.longhands.contains(id)
        } else {
            false  // For custom properties, always scan
        };


        if !definitely_new {
            let mut index_to_remove = None;
            for (i, slot) in self.declarations.iter_mut().enumerate() {
                if slot.0.id() == declaration.id() {
                    match (slot.1, importance) {
                        (Importance::Normal, Importance::Important) => {
                            self.important_count += 1;
                        }
                        (Importance::Important, Importance::Normal) => {
                            if overwrite_more_important_and_reuse_slot {
                                self.important_count -= 1;
                            } else {
                                return false
                            }
                        }
                        _ => if slot.0 == declaration {
                            return false;
                        }
                    }

                    if overwrite_more_important_and_reuse_slot {
                        *slot = (declaration, importance);
                        return true;
                    }

                    // NOTE(emilio): We could avoid this and just override for
                    // properties not affected by logical props, but it's not
                    // clear it's worth it given the `definitely_new` check.
                    index_to_remove = Some(i);
                    break;
                }
            }

            if let Some(index) = index_to_remove {
                self.declarations.remove(index);
                self.declarations.push((declaration, importance));
                return true;
            }
        }

        if let PropertyDeclarationId::Longhand(id) = declaration.id() {
            self.longhands.insert(id);
        }
        self.declarations.push((declaration, importance));
        if importance.important() {
            self.important_count += 1;
        }
        true
    }

    /// Set the declaration importance for a given property, if found.
    ///
    /// Returns whether any declaration was updated.
    pub fn set_importance(&mut self, property: &PropertyId, new_importance: Importance) -> bool {
        let mut updated_at_least_one = false;
        for &mut (ref declaration, ref mut importance) in &mut self.declarations {
            if declaration.id().is_or_is_longhand_of(property) {
                match (*importance, new_importance) {
                    (Importance::Normal, Importance::Important) => {
                        self.important_count += 1;
                    }
                    (Importance::Important, Importance::Normal) => {
                        self.important_count -= 1;
                    }
                    _ => {
                        continue;
                    }
                }
                updated_at_least_one = true;
                *importance = new_importance;
            }
        }
        updated_at_least_one
    }

    /// https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-removeproperty
    ///
    /// Returns whether any declaration was actually removed.
    pub fn remove_property(&mut self, property: &PropertyId) -> bool {
        if let PropertyId::Longhand(id) = *property {
            if !self.longhands.contains(id) {
                return false
            }
        }
        let important_count = &mut self.important_count;
        let mut removed_at_least_one = false;
        let longhands = &mut self.longhands;
        self.declarations.retain(|&(ref declaration, importance)| {
            let id = declaration.id();
            let remove = id.is_or_is_longhand_of(property);
            if remove {
                removed_at_least_one = true;
                if let PropertyDeclarationId::Longhand(id) = id {
                    longhands.remove(id)
                }
                if importance.important() {
                    *important_count -= 1
                }
            }
            !remove
        });

        if let PropertyId::Longhand(_) = *property {
            debug_assert!(removed_at_least_one);
        }
        removed_at_least_one
    }

    /// Take a declaration block known to contain a single property and serialize it.
    pub fn single_value_to_css<W>(&self, property: &PropertyId, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match property.as_shorthand() {
            Err(_longhand_or_custom) => {
                if self.declarations.len() == 1 {
                    self.declarations[0].0.to_css(dest)
                } else {
                    Err(fmt::Error)
                }
            }
            Ok(shorthand) => {
                if !self.declarations.iter().all(|decl| decl.0.shorthands().contains(&shorthand)) {
                    return Err(fmt::Error)
                }
                let iter = self.declarations_iter();
                match shorthand.get_shorthand_appendable_value(iter) {
                    Some(AppendableValue::Css { css, .. }) => {
                        dest.write_str(css)
                    },
                    Some(AppendableValue::DeclarationsForShorthand(_, decls)) => {
                        shorthand.longhands_to_css(decls, dest)
                    }
                    _ => Ok(())
                }
            }
        }
    }

    /// Convert AnimationValueMap to PropertyDeclarationBlock.
    #[cfg(feature = "gecko")]
    pub fn from_animation_value_map(animation_value_map: &AnimationValueMap) -> Self {
        let mut declarations = vec![];
        let mut longhands = LonghandIdSet::new();

        for (property, animation_value) in animation_value_map.iter() {
          longhands.set_animatable_longhand_bit(property);
          declarations.push((animation_value.uncompute(), Importance::Normal));
        }

        PropertyDeclarationBlock {
            declarations: declarations,
            important_count: 0,
            longhands: longhands,
        }
    }

    /// Returns true if the declaration block has a CSSWideKeyword for the given
    /// property.
    #[cfg(feature = "gecko")]
    pub fn has_css_wide_keyword(&self, property: &PropertyId) -> bool {
        if let PropertyId::Longhand(id) = *property {
            if !self.longhands.contains(id) {
                return false
            }
        }
        self.declarations.iter().any(|&(ref decl, _)|
            decl.id().is_or_is_longhand_of(property) &&
            decl.get_css_wide_keyword().is_some()
        )
    }
}

impl ToCss for PropertyDeclarationBlock {
    // https://drafts.csswg.org/cssom/#serialize-a-css-declaration-block
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        let mut is_first_serialization = true; // trailing serializations should have a prepended space

        // Step 1 -> dest = result list

        // Step 2
        let mut already_serialized = PropertyDeclarationIdSet::new();

        // Step 3
        for &(ref declaration, importance) in &*self.declarations {
            // Step 3.1
            let property = declaration.id();

            // Step 3.2
            if already_serialized.contains(property) {
                continue;
            }

            // Step 3.3
            let shorthands = declaration.shorthands();
            if !shorthands.is_empty() {
                // Step 3.3.1 is done by checking already_serialized while
                // iterating below.

                // Step 3.3.2
                for &shorthand in shorthands {
                    let properties = shorthand.longhands();

                    // Substep 2 & 3
                    let mut current_longhands = SmallVec::<[_; 10]>::new();
                    let mut important_count = 0;
                    let mut found_system = None;

                    let is_system_font =
                        shorthand == ShorthandId::Font &&
                        self.declarations.iter().any(|&(ref l, _)| {
                            !already_serialized.contains(l.id()) &&
                            l.get_system().is_some()
                        });

                    if is_system_font {
                        for &(ref longhand, longhand_importance) in &self.declarations {
                            if already_serialized.contains(longhand.id()) {
                                continue;
                            }

                            if longhand.get_system().is_some() || longhand.is_default_line_height() {
                                current_longhands.push(longhand);
                                if found_system.is_none() {
                                   found_system = longhand.get_system();
                                }
                                if longhand_importance.important() {
                                    important_count += 1;
                                }
                            }
                        }
                    } else {
                        for &(ref longhand, longhand_importance) in &self.declarations {
                            if already_serialized.contains(longhand.id()) {
                                continue;
                            }

                            if longhand.id().is_longhand_of(shorthand) {
                                current_longhands.push(longhand);
                                if longhand_importance.important() {
                                    important_count += 1;
                                }
                            }
                        }
                        // Substep 1:
                        //
                        // Assuming that the PropertyDeclarationBlock contains no
                        // duplicate entries, if the current_longhands length is
                        // equal to the properties length, it means that the
                        // properties that map to shorthand are present in longhands
                        if current_longhands.len() != properties.len() {
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
                    let mut v = String::new();
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
                            sys.to_css(&mut v)?;
                            AppendableValue::Css {
                                css: &v,
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
                                css: &v,
                                with_variables: false,
                            }
                        }
                    };

                    // Substeps 7 and 8
                    // We need to check the shorthand whether it's an alias property or not.
                    // If it's an alias property, it should be serialized like its longhand.
                    if shorthand.flags().contains(SHORTHAND_ALIAS_PROPERTY) {
                        append_serialization::<_, Cloned<slice::Iter< _>>, _>(
                             dest,
                             &property,
                             value,
                             importance,
                             &mut is_first_serialization)?;
                    } else {
                        append_serialization::<_, Cloned<slice::Iter< _>>, _>(
                             dest,
                             &shorthand,
                             value,
                             importance,
                             &mut is_first_serialization)?;
                    }

                    for current_longhand in &current_longhands {
                        // Substep 9
                        already_serialized.insert(current_longhand.id());
                    }
                }
            }

            // Step 3.3.4
            if already_serialized.contains(property) {
                continue;
            }

            use std::iter::Cloned;
            use std::slice;

            // Steps 3.3.5, 3.3.6 & 3.3.7
            // Need to specify an iterator type here even though it’s unused to work around
            // "error: unable to infer enough type information about `_`;
            //  type annotations or generic parameter binding required [E0282]"
            // Use the same type as earlier call to reuse generated code.
            append_serialization::<_, Cloned<slice::Iter<_>>, _>(
                dest,
                &property,
                AppendableValue::Declaration(declaration),
                importance,
                &mut is_first_serialization)?;

            // Step 3.3.8
            already_serialized.insert(property);
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
        css: &'a str,
        /// Whether the original serialization contained variables or not.
        with_variables: bool,
    }
}

/// Potentially appends whitespace after the first (property: value;) pair.
fn handle_first_serialization<W>(dest: &mut W,
                                 is_first_serialization: &mut bool)
                                 -> fmt::Result
    where W: fmt::Write,
{
    if !*is_first_serialization {
        dest.write_str(" ")
    } else {
        *is_first_serialization = false;
        Ok(())
    }
}

/// Append a given kind of appendable value to a serialization.
pub fn append_declaration_value<'a, W, I>(dest: &mut W,
                                          appendable_value: AppendableValue<'a, I>)
                                          -> fmt::Result
    where W: fmt::Write,
          I: Iterator<Item=&'a PropertyDeclaration>,
{
    match appendable_value {
        AppendableValue::Css { css, .. } => {
            dest.write_str(css)
        },
        AppendableValue::Declaration(decl) => {
            decl.to_css(dest)
        },
        AppendableValue::DeclarationsForShorthand(shorthand, decls) => {
            shorthand.longhands_to_css(decls, dest)
        }
    }
}

/// Append a given property and value pair to a serialization.
pub fn append_serialization<'a, W, I, N>(dest: &mut W,
                                         property_name: &N,
                                         appendable_value: AppendableValue<'a, I>,
                                         importance: Importance,
                                         is_first_serialization: &mut bool)
                                         -> fmt::Result
    where W: fmt::Write,
          I: Iterator<Item=&'a PropertyDeclaration>,
          N: ToCss,
{
    handle_first_serialization(dest, is_first_serialization)?;

    property_name.to_css(dest)?;
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
pub fn parse_style_attribute(input: &str,
                             url_data: &UrlExtraData,
                             error_reporter: &ParseErrorReporter,
                             quirks_mode: QuirksMode)
                             -> PropertyDeclarationBlock {
    let context = ParserContext::new(Origin::Author,
                                     url_data,
                                     error_reporter,
                                     Some(CssRuleType::Style),
                                     PARSING_MODE_DEFAULT,
                                     quirks_mode);
    let mut input = ParserInput::new(input);
    parse_property_declaration_list(&context, &mut Parser::new(&mut input))
}

/// Parse a given property declaration. Can result in multiple
/// `PropertyDeclaration`s when expanding a shorthand, for example.
///
/// This does not attempt to parse !important at all.
pub fn parse_one_declaration_into(declarations: &mut SourcePropertyDeclaration,
                                  id: PropertyId,
                                  input: &str,
                                  url_data: &UrlExtraData,
                                  error_reporter: &ParseErrorReporter,
                                  parsing_mode: ParsingMode,
                                  quirks_mode: QuirksMode)
                                  -> Result<(), ()> {
    let context = ParserContext::new(Origin::Author,
                                     url_data,
                                     error_reporter,
                                     Some(CssRuleType::Style),
                                     parsing_mode,
                                     quirks_mode);
    let mut input = ParserInput::new(input);
    let mut parser = Parser::new(&mut input);
    let start = parser.position();
    parser.parse_entirely(|parser| {
        PropertyDeclaration::parse_into(declarations, id, &context, parser)
            .map_err(|e| e.into())
    }).map_err(|err| {
        let end = parser.position();
        let error = ContextualParseError::UnsupportedPropertyDeclaration(
            parser.slice(start..end), err);
        log_css_error(&mut parser, start, error, &context);
    })
}

/// A struct to parse property declarations.
struct PropertyDeclarationParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
    declarations: &'a mut SourcePropertyDeclaration,
}


/// Default methods reject all at rules.
impl<'a, 'b, 'i> AtRuleParser<'i> for PropertyDeclarationParser<'a, 'b> {
    type Prelude = ();
    type AtRule = Importance;
    type Error = SelectorParseError<'i, StyleParseError<'i>>;
}

impl<'a, 'b, 'i> DeclarationParser<'i> for PropertyDeclarationParser<'a, 'b> {
    type Declaration = Importance;
    type Error = SelectorParseError<'i, StyleParseError<'i>>;

    fn parse_value<'t>(&mut self, name: CompactCowStr<'i>, input: &mut Parser<'i, 't>)
                       -> Result<Importance, ParseError<'i>> {
        let id = PropertyId::parse(name)?;
        input.parse_until_before(Delimiter::Bang, |input| {
            PropertyDeclaration::parse_into(self.declarations, id, self.context, input)
                .map_err(|e| e.into())
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
pub fn parse_property_declaration_list(context: &ParserContext,
                                       input: &mut Parser)
                                       -> PropertyDeclarationBlock {
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
                block.extend(iter.parser.declarations.drain(), importance);
            }
            Err(err) => {
                iter.parser.declarations.clear();
                let pos = err.span.start;
                let error = ContextualParseError::UnsupportedPropertyDeclaration(
                    iter.input.slice(err.span), err.error);
                log_css_error(iter.input, pos, error, &context);
            }
        }
    }
    block
}
