/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A property declaration block.

#![deny(missing_docs)]

use super::*;
use crate::applicable_declarations::CascadePriority;
use crate::context::QuirksMode;
use crate::custom_properties::{self, CustomPropertiesBuilder};
use crate::error_reporting::{ContextualParseError, ParseErrorReporter};
use crate::parser::ParserContext;
use crate::properties::animated_properties::{AnimationValue, AnimationValueMap};
use crate::rule_tree::CascadeLevel;
use crate::selector_map::PrecomputedHashSet;
use crate::selector_parser::SelectorImpl;
use crate::shared_lock::Locked;
use crate::str::{CssString, CssStringWriter};
use crate::stylesheets::{layer_rule::LayerOrder, CssRuleType, Origin, UrlExtraData};
use crate::values::computed::Context;
use cssparser::{
    parse_important, AtRuleParser, CowRcStr, DeclarationListParser, DeclarationParser, Delimiter,
    ParseErrorKind, Parser, ParserInput, QualifiedRuleParser,
};
use itertools::Itertools;
use selectors::SelectorList;
use smallbitvec::{self, SmallBitVec};
use smallvec::SmallVec;
use std::fmt::{self, Write};
use std::iter::{DoubleEndedIterator, Zip};
use std::slice::Iter;
use style_traits::{CssWriter, ParseError, ParsingMode, StyleParseErrorKind, ToCss};
use thin_vec::ThinVec;

/// A set of property declarations including animations and transitions.
#[derive(Default)]
pub struct AnimationDeclarations {
    /// Declarations for animations.
    pub animations: Option<Arc<Locked<PropertyDeclarationBlock>>>,
    /// Declarations for transitions.
    pub transitions: Option<Arc<Locked<PropertyDeclarationBlock>>>,
}

impl AnimationDeclarations {
    /// Whether or not this `AnimationDeclarations` is empty.
    pub fn is_empty(&self) -> bool {
        self.animations.is_none() && self.transitions.is_none()
    }
}

/// An enum describes how a declaration should update
/// the declaration block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeclarationUpdate {
    /// The given declaration doesn't update anything.
    None,
    /// The given declaration is new, and should be append directly.
    Append,
    /// The given declaration can be updated in-place at the given position.
    UpdateInPlace { pos: usize },
    /// The given declaration cannot be updated in-place, and an existing
    /// one needs to be removed at the given position.
    AppendAndRemove { pos: usize },
}

/// A struct describes how a declaration block should be updated by
/// a `SourcePropertyDeclaration`.
#[derive(Default)]
pub struct SourcePropertyDeclarationUpdate {
    updates: SubpropertiesVec<DeclarationUpdate>,
    new_count: usize,
    any_removal: bool,
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

#[derive(Clone, ToShmem, Default, MallocSizeOf)]
struct PropertyDeclarationIdSet {
    longhands: LonghandIdSet,
    custom: PrecomputedHashSet<custom_properties::Name>,
}

impl PropertyDeclarationIdSet {
    fn insert(&mut self, id: PropertyDeclarationId) -> bool {
        match id {
            PropertyDeclarationId::Longhand(id) => {
                if self.longhands.contains(id) {
                    return false;
                }
                self.longhands.insert(id);
                return true;
            },
            PropertyDeclarationId::Custom(name) => self.custom.insert(name.clone()),
        }
    }

    fn contains(&self, id: PropertyDeclarationId) -> bool {
        match id {
            PropertyDeclarationId::Longhand(id) => self.longhands.contains(id),
            PropertyDeclarationId::Custom(name) => self.custom.contains(name),
        }
    }

    fn remove(&mut self, id: PropertyDeclarationId) {
        match id {
            PropertyDeclarationId::Longhand(id) => self.longhands.remove(id),
            PropertyDeclarationId::Custom(name) => {
                self.custom.remove(name);
            },
        }
    }

    fn clear(&mut self) {
        self.longhands.clear();
        self.custom.clear();
    }
}

/// Overridden declarations are skipped.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, ToShmem)]
pub struct PropertyDeclarationBlock {
    /// The group of declarations, along with their importance.
    ///
    /// Only deduplicated declarations appear here.
    declarations: ThinVec<PropertyDeclaration>,

    /// The "important" flag for each declaration in `declarations`.
    declarations_importance: SmallBitVec,

    /// The set of properties that are present in the block.
    property_ids: PropertyDeclarationIdSet,
}

/// Iterator over `(PropertyDeclaration, Importance)` pairs.
pub struct DeclarationImportanceIterator<'a> {
    iter: Zip<Iter<'a, PropertyDeclaration>, smallbitvec::Iter<'a>>,
}

impl<'a> Default for DeclarationImportanceIterator<'a> {
    fn default() -> Self {
        Self {
            iter: [].iter().zip(smallbitvec::Iter::default()),
        }
    }
}

impl<'a> DeclarationImportanceIterator<'a> {
    /// Constructor.
    fn new(declarations: &'a [PropertyDeclaration], important: &'a SmallBitVec) -> Self {
        DeclarationImportanceIterator {
            iter: declarations.iter().zip(important.iter()),
        }
    }
}

impl<'a> Iterator for DeclarationImportanceIterator<'a> {
    type Item = (&'a PropertyDeclaration, Importance);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(decl, important)| {
            (
                decl,
                if important {
                    Importance::Important
                } else {
                    Importance::Normal
                },
            )
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator for DeclarationImportanceIterator<'a> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(decl, important)| {
            (
                decl,
                if important {
                    Importance::Important
                } else {
                    Importance::Normal
                },
            )
        })
    }
}

/// Iterator for AnimationValue to be generated from PropertyDeclarationBlock.
pub struct AnimationValueIterator<'a, 'cx, 'cx_a: 'cx> {
    iter: DeclarationImportanceIterator<'a>,
    context: &'cx mut Context<'cx_a>,
    default_values: &'a ComputedValues,
    /// Custom properties in a keyframe if exists.
    extra_custom_properties: Option<&'a Arc<crate::custom_properties::CustomPropertiesMap>>,
}

impl<'a, 'cx, 'cx_a: 'cx> AnimationValueIterator<'a, 'cx, 'cx_a> {
    fn new(
        declarations: &'a PropertyDeclarationBlock,
        context: &'cx mut Context<'cx_a>,
        default_values: &'a ComputedValues,
        extra_custom_properties: Option<&'a Arc<crate::custom_properties::CustomPropertiesMap>>,
    ) -> AnimationValueIterator<'a, 'cx, 'cx_a> {
        AnimationValueIterator {
            iter: declarations.declaration_importance_iter(),
            context,
            default_values,
            extra_custom_properties,
        }
    }
}

impl<'a, 'cx, 'cx_a: 'cx> Iterator for AnimationValueIterator<'a, 'cx, 'cx_a> {
    type Item = AnimationValue;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (decl, importance) = self.iter.next()?;

            if importance.important() {
                continue;
            }

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
            declarations: ThinVec::new(),
            declarations_importance: SmallBitVec::new(),
            property_ids: PropertyDeclarationIdSet::default(),
        }
    }

    /// Create a block with a single declaration
    pub fn with_one(declaration: PropertyDeclaration, importance: Importance) -> Self {
        let mut property_ids = PropertyDeclarationIdSet::default();
        property_ids.insert(declaration.id());
        let mut declarations = ThinVec::with_capacity(1);
        declarations.push(declaration);
        PropertyDeclarationBlock {
            declarations,
            declarations_importance: SmallBitVec::from_elem(1, importance.important()),
            property_ids,
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
    pub fn normal_declaration_iter<'a>(
        &'a self,
    ) -> impl DoubleEndedIterator<Item = &'a PropertyDeclaration> {
        self.declaration_importance_iter()
            .filter(|(_, importance)| !importance.important())
            .map(|(declaration, _)| declaration)
    }

    /// Return an iterator of (AnimatableLonghand, AnimationValue).
    #[inline]
    pub fn to_animation_value_iter<'a, 'cx, 'cx_a: 'cx>(
        &'a self,
        context: &'cx mut Context<'cx_a>,
        default_values: &'a ComputedValues,
        extra_custom_properties: Option<&'a Arc<crate::custom_properties::CustomPropertiesMap>>,
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

    /// Returns a `LonghandIdSet` representing the properties that are changed in
    /// this block.
    #[inline]
    pub fn longhands(&self) -> &LonghandIdSet {
        &self.property_ids.longhands
    }

    /// Returns whether this block contains a declaration of a given property id.
    #[inline]
    pub fn contains(&self, id: PropertyDeclarationId) -> bool {
        self.property_ids.contains(id)
    }

    /// Returns whether this block contains any reset longhand.
    #[inline]
    pub fn contains_any_reset(&self) -> bool {
        self.property_ids.longhands.contains_any_reset()
    }

    /// Get a declaration for a given property.
    ///
    /// NOTE: This is linear time in the case of custom properties or in the
    /// case the longhand is actually in the declaration block.
    #[inline]
    pub fn get(
        &self,
        property: PropertyDeclarationId,
    ) -> Option<(&PropertyDeclaration, Importance)> {
        if !self.contains(property) {
            return None;
        }
        self.declaration_importance_iter()
            .find(|(declaration, _)| declaration.id() == property)
    }

    /// Tries to serialize a given shorthand from the declarations in this
    /// block.
    pub fn shorthand_to_css(
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
        match shorthand.get_shorthand_appendable_value(&list) {
            Some(appendable_value) => append_declaration_value(dest, appendable_value),
            None => return Ok(()),
        }
    }

    /// Find the value of the given property in this block and serialize it
    ///
    /// <https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-getpropertyvalue>
    pub fn property_value_to_css(
        &self,
        property: &PropertyId,
        dest: &mut CssStringWriter,
    ) -> fmt::Result {
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
            },
            Err(longhand_or_custom) => {
                // Step 3
                self.get(longhand_or_custom)
                    .map_or(Importance::Normal, |(_, importance)| importance)
            },
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
    ) -> bool {
        let all_shorthand_len = match drain.all_shorthand {
            AllShorthand::NotSet => 0,
            AllShorthand::CSSWideKeyword(_) | AllShorthand::WithVariables(_) => {
                shorthands::ALL_SHORTHAND_MAX_LEN
            },
        };
        let push_calls_count = drain.declarations.len() + all_shorthand_len;

        // With deduplication the actual length increase may be less than this.
        self.declarations.reserve(push_calls_count);

        let mut changed = false;
        for decl in &mut drain.declarations {
            changed |= self.push(decl, importance);
        }
        drain
            .all_shorthand
            .declarations()
            .fold(changed, |changed, decl| {
                changed | self.push(decl, importance)
            })
    }

    /// Adds or overrides the declaration for a given property in this block.
    ///
    /// Returns whether the declaration has changed.
    ///
    /// This is only used for parsing and internal use.
    pub fn push(&mut self, declaration: PropertyDeclaration, importance: Importance) -> bool {
        let id = declaration.id();
        if !self.property_ids.insert(id) {
            let mut index_to_remove = None;
            for (i, slot) in self.declarations.iter_mut().enumerate() {
                if slot.id() != id {
                    continue;
                }

                let important = self.declarations_importance[i];

                // For declarations from parsing, non-important declarations
                // shouldn't override existing important one.
                if important && !importance.important() {
                    return false;
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

        self.declarations.push(declaration);
        self.declarations_importance.push(importance.important());
        true
    }

    /// Prepares updating this declaration block with the given
    /// `SourcePropertyDeclaration` and importance, and returns whether
    /// there is something to update.
    pub fn prepare_for_update(
        &self,
        source_declarations: &SourcePropertyDeclaration,
        importance: Importance,
        updates: &mut SourcePropertyDeclarationUpdate,
    ) -> bool {
        debug_assert!(updates.updates.is_empty());
        // Check whether we are updating for an all shorthand change.
        if !matches!(source_declarations.all_shorthand, AllShorthand::NotSet) {
            debug_assert!(source_declarations.declarations.is_empty());
            return source_declarations
                .all_shorthand
                .declarations()
                .any(|decl| {
                    !self.contains(decl.id()) ||
                        self.declarations
                            .iter()
                            .enumerate()
                            .find(|&(_, ref d)| d.id() == decl.id())
                            .map_or(true, |(i, d)| {
                                let important = self.declarations_importance[i];
                                *d != decl || important != importance.important()
                            })
                });
        }
        // Fill `updates` with update information.
        let mut any_update = false;
        let new_count = &mut updates.new_count;
        let any_removal = &mut updates.any_removal;
        let updates = &mut updates.updates;
        updates.extend(
            source_declarations
                .declarations
                .iter()
                .map(|declaration| {
                    if !self.contains(declaration.id()) {
                        return DeclarationUpdate::Append;
                    }
                    let longhand_id = declaration.id().as_longhand();
                    if let Some(longhand_id) = longhand_id {
                        if let Some(logical_group) = longhand_id.logical_group() {
                            let mut needs_append = false;
                            for (pos, decl) in self.declarations.iter().enumerate().rev() {
                                let id = match decl.id().as_longhand() {
                                    Some(id) => id,
                                    None => continue,
                                };
                                if id == longhand_id {
                                    if needs_append {
                                        return DeclarationUpdate::AppendAndRemove { pos };
                                    }
                                    let important = self.declarations_importance[pos];
                                    if decl == declaration && important == importance.important() {
                                        return DeclarationUpdate::None;
                                    }
                                    return DeclarationUpdate::UpdateInPlace { pos };
                                }
                                if !needs_append &&
                                    id.logical_group() == Some(logical_group) &&
                                    id.is_logical() != longhand_id.is_logical()
                                {
                                    needs_append = true;
                                }
                            }
                            unreachable!("Longhand should be found in loop above");
                        }
                    }
                    self.declarations
                        .iter()
                        .enumerate()
                        .find(|&(_, ref decl)| decl.id() == declaration.id())
                        .map_or(DeclarationUpdate::Append, |(pos, decl)| {
                            let important = self.declarations_importance[pos];
                            if decl == declaration && important == importance.important() {
                                DeclarationUpdate::None
                            } else {
                                DeclarationUpdate::UpdateInPlace { pos }
                            }
                        })
                })
                .inspect(|update| {
                    if matches!(update, DeclarationUpdate::None) {
                        return;
                    }
                    any_update = true;
                    match update {
                        DeclarationUpdate::Append => {
                            *new_count += 1;
                        },
                        DeclarationUpdate::AppendAndRemove { .. } => {
                            *any_removal = true;
                        },
                        _ => {},
                    }
                }),
        );
        any_update
    }

    /// Update this declaration block with the given data.
    pub fn update(
        &mut self,
        drain: SourcePropertyDeclarationDrain,
        importance: Importance,
        updates: &mut SourcePropertyDeclarationUpdate,
    ) {
        let important = importance.important();
        if !matches!(drain.all_shorthand, AllShorthand::NotSet) {
            debug_assert!(updates.updates.is_empty());
            for decl in drain.all_shorthand.declarations() {
                let id = decl.id();
                if self.property_ids.insert(id) {
                    self.declarations.push(decl);
                    self.declarations_importance.push(important);
                } else {
                    let (idx, slot) = self
                        .declarations
                        .iter_mut()
                        .enumerate()
                        .find(|&(_, ref d)| d.id() == decl.id())
                        .unwrap();
                    *slot = decl;
                    self.declarations_importance.set(idx, important);
                }
            }
            return;
        }

        self.declarations.reserve(updates.new_count);
        if updates.any_removal {
            // Prepare for removal and fixing update positions.
            struct UpdateOrRemoval<'a> {
                item: &'a mut DeclarationUpdate,
                pos: usize,
                remove: bool,
            }
            let mut updates_and_removals: SubpropertiesVec<UpdateOrRemoval> = updates
                .updates
                .iter_mut()
                .filter_map(|item| {
                    let (pos, remove) = match *item {
                        DeclarationUpdate::UpdateInPlace { pos } => (pos, false),
                        DeclarationUpdate::AppendAndRemove { pos } => (pos, true),
                        _ => return None,
                    };
                    Some(UpdateOrRemoval { item, pos, remove })
                })
                .collect();
            // Execute removals. It's important to do it in reverse index order,
            // so that removing doesn't invalidate following positions.
            updates_and_removals.sort_unstable_by_key(|update| update.pos);
            updates_and_removals
                .iter()
                .rev()
                .filter(|update| update.remove)
                .for_each(|update| {
                    self.declarations.remove(update.pos);
                    self.declarations_importance.remove(update.pos);
                });
            // Fixup pos field for updates.
            let mut removed_count = 0;
            for update in updates_and_removals.iter_mut() {
                if update.remove {
                    removed_count += 1;
                    continue;
                }
                debug_assert_eq!(
                    *update.item,
                    DeclarationUpdate::UpdateInPlace { pos: update.pos }
                );
                *update.item = DeclarationUpdate::UpdateInPlace {
                    pos: update.pos - removed_count,
                };
            }
        }
        // Execute updates and appends.
        for (decl, update) in drain.declarations.zip_eq(updates.updates.iter()) {
            match *update {
                DeclarationUpdate::None => {},
                DeclarationUpdate::Append | DeclarationUpdate::AppendAndRemove { .. } => {
                    self.property_ids.insert(decl.id());
                    self.declarations.push(decl);
                    self.declarations_importance.push(important);
                },
                DeclarationUpdate::UpdateInPlace { pos } => {
                    self.declarations[pos] = decl;
                    self.declarations_importance.set(pos, important);
                },
            }
        }
        updates.updates.clear();
    }

    /// Returns the first declaration that would be removed by removing
    /// `property`.
    #[inline]
    pub fn first_declaration_to_remove(&self, property: &PropertyId) -> Option<usize> {
        if let Err(longhand_or_custom) = property.as_shorthand() {
            if !self.contains(longhand_or_custom) {
                return None;
            }
        }

        self.declarations
            .iter()
            .position(|declaration| declaration.id().is_or_is_longhand_of(property))
    }

    /// Removes a given declaration at a given index.
    #[inline]
    fn remove_declaration_at(&mut self, i: usize) {
        self.property_ids.remove(self.declarations[i].id());
        self.declarations_importance.remove(i);
        self.declarations.remove(i);
    }

    /// Clears all the declarations from this block.
    #[inline]
    pub fn clear(&mut self) {
        self.declarations_importance.clear();
        self.declarations.clear();
        self.property_ids.clear();
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-removeproperty>
    ///
    /// `first_declaration` needs to be the result of
    /// `first_declaration_to_remove`.
    #[inline]
    pub fn remove_property(&mut self, property: &PropertyId, first_declaration: usize) {
        debug_assert_eq!(
            Some(first_declaration),
            self.first_declaration_to_remove(property)
        );
        debug_assert!(self.declarations[first_declaration]
            .id()
            .is_or_is_longhand_of(property));

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
        device: &Device,
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
                block.cascade_custom_properties(cv.custom_properties(), device)
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
            (&PropertyDeclaration::WithVariables(ref declaration), Some(ref computed_values)) => {
                declaration
                    .value
                    .substitute_variables(
                        declaration.id,
                        computed_values.writing_mode,
                        custom_properties.as_ref(),
                        QuirksMode::NoQuirks,
                        device,
                        &mut Default::default(),
                    )
                    .to_css(dest)
            },
            (ref d, _) => d.to_css(dest),
        }
    }

    /// Convert AnimationValueMap to PropertyDeclarationBlock.
    pub fn from_animation_value_map(animation_value_map: &AnimationValueMap) -> Self {
        let len = animation_value_map.len();
        let mut declarations = ThinVec::with_capacity(len);
        let mut property_ids = PropertyDeclarationIdSet::default();

        for (property, animation_value) in animation_value_map.iter() {
            property_ids.longhands.insert(*property);
            declarations.push(animation_value.uncompute());
        }

        PropertyDeclarationBlock {
            declarations,
            property_ids,
            declarations_importance: SmallBitVec::from_elem(len, false),
        }
    }

    /// Returns true if the declaration block has a CSSWideKeyword for the given
    /// property.
    pub fn has_css_wide_keyword(&self, property: &PropertyId) -> bool {
        if let Err(longhand_or_custom) = property.as_shorthand() {
            if !self.property_ids.contains(longhand_or_custom) {
                return false;
            }
        }
        self.declarations.iter().any(|decl| {
            decl.id().is_or_is_longhand_of(property) && decl.get_css_wide_keyword().is_some()
        })
    }

    /// Returns a custom properties map which is the result of cascading custom
    /// properties in this declaration block along with context's custom
    /// properties.
    pub fn cascade_custom_properties_with_context(
        &self,
        context: &Context,
    ) -> Option<Arc<crate::custom_properties::CustomPropertiesMap>> {
        self.cascade_custom_properties(context.style().custom_properties(), context.device())
    }

    /// Returns a custom properties map which is the result of cascading custom
    /// properties in this declaration block along with the given custom
    /// properties.
    fn cascade_custom_properties(
        &self,
        inherited_custom_properties: Option<&Arc<crate::custom_properties::CustomPropertiesMap>>,
        device: &Device,
    ) -> Option<Arc<crate::custom_properties::CustomPropertiesMap>> {
        let mut builder = CustomPropertiesBuilder::new(inherited_custom_properties, device);

        for declaration in self.normal_declaration_iter() {
            if let PropertyDeclaration::Custom(ref declaration) = *declaration {
                builder.cascade(
                    declaration,
                    CascadePriority::new(
                        CascadeLevel::same_tree_author_normal(),
                        LayerOrder::root(),
                    ),
                );
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
        'declaration_loop: for (declaration, importance) in self.declaration_importance_iter() {
            // Step 3.1
            let property = declaration.id();
            let longhand_id = match property {
                PropertyDeclarationId::Longhand(id) => id,
                PropertyDeclarationId::Custom(..) => {
                    // Given the invariants that there are no duplicate
                    // properties in a declaration block, and that custom
                    // properties can't be part of a shorthand, we can just care
                    // about them here.
                    append_serialization(
                        dest,
                        &property,
                        AppendableValue::Declaration(declaration),
                        importance,
                        &mut is_first_serialization,
                    )?;
                    continue;
                },
            };

            // Step 3.2
            if already_serialized.contains(longhand_id.into()) {
                continue;
            }

            // Steps 3.3 & 3.4
            for shorthand in longhand_id.shorthands() {
                // We already attempted to serialize this shorthand before.
                if already_serialized.contains(shorthand.into()) {
                    continue;
                }
                already_serialized.insert(shorthand.into());

                if shorthand.is_legacy_shorthand() {
                    continue;
                }

                // Step 3.3.1:
                //     Let longhands be an array consisting of all CSS
                //     declarations in declaration block’s declarations that
                //     that are not in already serialized and have a property
                //     name that maps to one of the shorthand properties in
                //     shorthands.
                let longhands = {
                    // TODO(emilio): This could just index in an array if we
                    // remove pref-controlled longhands.
                    let mut ids = LonghandIdSet::new();
                    for longhand in shorthand.longhands() {
                        ids.insert(longhand);
                    }
                    ids
                };

                // Step 3.4.2
                //     If all properties that map to shorthand are not present
                //     in longhands, continue with the steps labeled shorthand
                //     loop.
                if !self.property_ids.longhands.contains_all(&longhands) {
                    continue;
                }

                // Step 3.4.3:
                //     Let current longhands be an empty array.
                let mut current_longhands = SmallVec::<[&_; 10]>::new();
                let mut logical_groups = LogicalGroupSet::new();
                let mut saw_one = false;
                let mut logical_mismatch = false;
                let mut seen = LonghandIdSet::new();
                let mut important_count = 0;

                // Step 3.4.4:
                //    Append all CSS declarations in longhands that have a
                //    property name that maps to shorthand to current longhands.
                for (declaration, importance) in self.declaration_importance_iter() {
                    let longhand = match declaration.id() {
                        PropertyDeclarationId::Longhand(id) => id,
                        PropertyDeclarationId::Custom(..) => continue,
                    };

                    if longhands.contains(longhand) {
                        saw_one = true;
                        if importance.important() {
                            important_count += 1;
                        }
                        current_longhands.push(declaration);
                        if shorthand != ShorthandId::All {
                            // All is special because it contains both physical
                            // and logical longhands.
                            if let Some(g) = longhand.logical_group() {
                                logical_groups.insert(g);
                            }
                            seen.insert(longhand);
                            if seen == longhands {
                                break;
                            }
                        }
                    } else if saw_one {
                        if let Some(g) = longhand.logical_group() {
                            if logical_groups.contains(g) {
                                logical_mismatch = true;
                                break;
                            }
                        }
                    }
                }

                // 3.4.5:
                //     If there is one or more CSS declarations in current
                //     longhands have their important flag set and one or more
                //     with it unset, continue with the steps labeled shorthand
                //     loop.
                let is_important = important_count > 0;
                if is_important && important_count != current_longhands.len() {
                    continue;
                }

                // 3.4.6:
                //    If there’s any declaration in declaration block in between
                //    the first and the last longhand in current longhands which
                //    belongs to the same logical property group, but has a
                //    different mapping logic as any of the longhands in current
                //    longhands, and is not in current longhands, continue with
                //    the steps labeled shorthand loop.
                if logical_mismatch {
                    continue;
                }

                let importance = if is_important {
                    Importance::Important
                } else {
                    Importance::Normal
                };

                // 3.4.7:
                //    Let value be the result of invoking serialize a CSS value
                //    of current longhands.
                let appendable_value =
                    match shorthand.get_shorthand_appendable_value(&current_longhands) {
                        None => continue,
                        Some(appendable_value) => appendable_value,
                    };

                // We avoid re-serializing if we're already an
                // AppendableValue::Css.
                let mut v = CssString::new();
                let value = match appendable_value {
                    AppendableValue::Css(css) => {
                        debug_assert!(!css.is_empty());
                        appendable_value
                    },
                    other => {
                        append_declaration_value(&mut v, other)?;

                        // 3.4.8:
                        //     If value is the empty string, continue with the
                        //     steps labeled shorthand loop.
                        if v.is_empty() {
                            continue;
                        }

                        AppendableValue::Css({
                            // Safety: serialization only generates valid utf-8.
                            #[cfg(feature = "gecko")]
                            unsafe {
                                v.as_str_unchecked()
                            }
                            #[cfg(feature = "servo")]
                            &v
                        })
                    },
                };

                // 3.4.9:
                //     Let serialized declaration be the result of invoking
                //     serialize a CSS declaration with property name shorthand,
                //     value value, and the important flag set if the CSS
                //     declarations in current longhands have their important
                //     flag set.
                //
                // 3.4.10:
                //     Append serialized declaration to list.
                append_serialization(
                    dest,
                    &shorthand,
                    value,
                    importance,
                    &mut is_first_serialization,
                )?;

                // 3.4.11:
                //     Append the property names of all items of current
                //     longhands to already serialized.
                for current_longhand in &current_longhands {
                    let longhand_id = match current_longhand.id() {
                        PropertyDeclarationId::Longhand(id) => id,
                        PropertyDeclarationId::Custom(..) => unreachable!(),
                    };

                    // Substep 9
                    already_serialized.insert(longhand_id.into());
                }

                // 3.4.12:
                //     Continue with the steps labeled declaration loop.
                continue 'declaration_loop;
            }

            // Steps 3.5, 3.6 & 3.7:
            //     Let value be the result of invoking serialize a CSS value of
            //     declaration.
            //
            //     Let serialized declaration be the result of invoking
            //     serialize a CSS declaration with property name property,
            //     value value, and the important flag set if declaration has
            //     its important flag set.
            //
            //     Append serialized declaration to list.
            append_serialization(
                dest,
                &property,
                AppendableValue::Declaration(declaration),
                importance,
                &mut is_first_serialization,
            )?;

            // Step 3.8:
            //     Append property to already serialized.
            already_serialized.insert(longhand_id.into());
        }

        // Step 4
        Ok(())
    }
}

/// A convenient enum to represent different kinds of stuff that can represent a
/// _value_ in the serialization of a property declaration.
pub enum AppendableValue<'a, 'b: 'a> {
    /// A given declaration, of which we'll serialize just the value.
    Declaration(&'a PropertyDeclaration),
    /// A set of declarations for a given shorthand.
    ///
    /// FIXME: This needs more docs, where are the shorthands expanded? We print
    /// the property name before-hand, don't we?
    DeclarationsForShorthand(ShorthandId, &'a [&'b PropertyDeclaration]),
    /// A raw CSS string, coming for example from a property with CSS variables,
    /// or when storing a serialized shorthand value before appending directly.
    Css(&'a str),
}

/// Potentially appends whitespace after the first (property: value;) pair.
fn handle_first_serialization<W>(dest: &mut W, is_first_serialization: &mut bool) -> fmt::Result
where
    W: Write,
{
    if !*is_first_serialization {
        dest.write_char(' ')
    } else {
        *is_first_serialization = false;
        Ok(())
    }
}

/// Append a given kind of appendable value to a serialization.
pub fn append_declaration_value<'a, 'b: 'a>(
    dest: &mut CssStringWriter,
    appendable_value: AppendableValue<'a, 'b>,
) -> fmt::Result {
    match appendable_value {
        AppendableValue::Css(css) => dest.write_str(css),
        AppendableValue::Declaration(decl) => decl.to_css(dest),
        AppendableValue::DeclarationsForShorthand(shorthand, decls) => {
            shorthand.longhands_to_css(decls, dest)
        },
    }
}

/// Append a given property and value pair to a serialization.
pub fn append_serialization<'a, 'b: 'a, N>(
    dest: &mut CssStringWriter,
    property_name: &N,
    appendable_value: AppendableValue<'a, 'b>,
    importance: Importance,
    is_first_serialization: &mut bool,
) -> fmt::Result
where
    N: ToCss,
{
    handle_first_serialization(dest, is_first_serialization)?;

    property_name.to_css(&mut CssWriter::new(dest))?;
    dest.write_str(": ")?;

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
    error_reporter: Option<&dyn ParseErrorReporter>,
    quirks_mode: QuirksMode,
    rule_type: CssRuleType,
) -> PropertyDeclarationBlock {
    let context = ParserContext::new(
        Origin::Author,
        url_data,
        Some(rule_type),
        ParsingMode::DEFAULT,
        quirks_mode,
        error_reporter,
        None,
    );

    let mut input = ParserInput::new(input);
    parse_property_declaration_list(&context, &mut Parser::new(&mut input), None)
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
    origin: Origin,
    url_data: &UrlExtraData,
    error_reporter: Option<&dyn ParseErrorReporter>,
    parsing_mode: ParsingMode,
    quirks_mode: QuirksMode,
    rule_type: CssRuleType,
) -> Result<(), ()> {
    let context = ParserContext::new(
        origin,
        url_data,
        Some(rule_type),
        parsing_mode,
        quirks_mode,
        error_reporter,
        None,
    );

    let property_id_for_error_reporting = if context.error_reporting_enabled() {
        Some(id.clone())
    } else {
        None
    };

    let mut input = ParserInput::new(input);
    let mut parser = Parser::new(&mut input);
    let start_position = parser.position();
    parser
        .parse_entirely(|parser| {
            PropertyDeclaration::parse_into(declarations, id, &context, parser)
        })
        .map_err(|err| {
            if context.error_reporting_enabled() {
                report_one_css_error(
                    &context,
                    None,
                    None,
                    err,
                    parser.slice_from(start_position),
                    property_id_for_error_reporting,
                )
            }
        })
}

/// A struct to parse property declarations.
struct PropertyDeclarationParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
    declarations: &'a mut SourcePropertyDeclaration,
    /// The last parsed property id if any.
    last_parsed_property_id: Option<PropertyId>,
}

/// Default methods reject all at rules.
impl<'a, 'b, 'i> AtRuleParser<'i> for PropertyDeclarationParser<'a, 'b> {
    type Prelude = ();
    type AtRule = Importance;
    type Error = StyleParseErrorKind<'i>;
}

/// Default methods reject all rules.
impl<'a, 'b, 'i> QualifiedRuleParser<'i> for PropertyDeclarationParser<'a, 'b> {
    type Prelude = ();
    type QualifiedRule = Importance;
    type Error = StyleParseErrorKind<'i>;
}

/// Based on NonMozillaVendorIdentifier from Gecko's CSS parser.
fn is_non_mozilla_vendor_identifier(name: &str) -> bool {
    (name.starts_with("-") && !name.starts_with("-moz-")) || name.starts_with("_")
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
                return Err(input.new_custom_error(StyleParseErrorKind::UnknownProperty(name)));
            },
        };
        if self.context.error_reporting_enabled() {
            self.last_parsed_property_id = Some(id.clone());
        }
        input.parse_until_before(Delimiter::Bang, |input| {
            PropertyDeclaration::parse_into(self.declarations, id, self.context, input)
        })?;
        let importance = match input.try_parse(parse_important) {
            Ok(()) => Importance::Important,
            Err(_) => Importance::Normal,
        };
        // In case there is still unparsed text in the declaration, we should roll back.
        input.expect_exhausted()?;
        Ok(importance)
    }
}

type SmallParseErrorVec<'i> = SmallVec<[(ParseError<'i>, &'i str, Option<PropertyId>); 2]>;

fn alias_of_known_property(name: &str) -> Option<PropertyId> {
    let mut prefixed = String::with_capacity(name.len() + 5);
    prefixed.push_str("-moz-");
    prefixed.push_str(name);
    PropertyId::parse_enabled_for_all_content(&prefixed).ok()
}

#[cold]
fn report_one_css_error<'i>(
    context: &ParserContext,
    block: Option<&PropertyDeclarationBlock>,
    selectors: Option<&SelectorList<SelectorImpl>>,
    mut error: ParseError<'i>,
    slice: &str,
    property: Option<PropertyId>,
) {
    debug_assert!(context.error_reporting_enabled());

    fn all_properties_in_block(block: &PropertyDeclarationBlock, property: &PropertyId) -> bool {
        match property.as_shorthand() {
            Ok(id) => id
                .longhands()
                .all(|longhand| block.contains(PropertyDeclarationId::Longhand(longhand))),
            Err(longhand_or_custom) => block.contains(longhand_or_custom),
        }
    }

    if let ParseErrorKind::Custom(StyleParseErrorKind::UnknownProperty(ref name)) = error.kind {
        if is_non_mozilla_vendor_identifier(name) {
            // If the unrecognized property looks like a vendor-specific property,
            // silently ignore it instead of polluting the error output.
            return;
        }
        if let Some(alias) = alias_of_known_property(name) {
            // This is an unknown property, but its -moz-* version is known.
            // We don't want to report error if the -moz-* version is already
            // specified.
            if let Some(block) = block {
                if all_properties_in_block(block, &alias) {
                    return;
                }
            }
        }
    }

    if let Some(ref property) = property {
        if let Some(block) = block {
            if all_properties_in_block(block, property) {
                return;
            }
        }
        error = match *property {
            PropertyId::Custom(ref c) => {
                StyleParseErrorKind::new_invalid(format!("--{}", c), error)
            },
            _ => StyleParseErrorKind::new_invalid(property.non_custom_id().unwrap().name(), error),
        };
    }

    let location = error.location;
    let error = ContextualParseError::UnsupportedPropertyDeclaration(slice, error, selectors);
    context.log_css_error(location, error);
}

#[cold]
fn report_css_errors(
    context: &ParserContext,
    block: &PropertyDeclarationBlock,
    selectors: Option<&SelectorList<SelectorImpl>>,
    errors: &mut SmallParseErrorVec,
) {
    for (error, slice, property) in errors.drain(..) {
        report_one_css_error(context, Some(block), selectors, error, slice, property)
    }
}

/// Parse a list of property declarations and return a property declaration
/// block.
pub fn parse_property_declaration_list(
    context: &ParserContext,
    input: &mut Parser,
    selectors: Option<&SelectorList<SelectorImpl>>,
) -> PropertyDeclarationBlock {
    let mut declarations = SourcePropertyDeclaration::new();
    let mut block = PropertyDeclarationBlock::new();
    let parser = PropertyDeclarationParser {
        context,
        last_parsed_property_id: None,
        declarations: &mut declarations,
    };
    let mut iter = DeclarationListParser::new(input, parser);
    let mut errors = SmallParseErrorVec::new();
    while let Some(declaration) = iter.next() {
        match declaration {
            Ok(importance) => {
                block.extend(iter.parser.declarations.drain(), importance);
                // We've successfully parsed a declaration, so forget about
                // `last_parsed_property_id`. It'd be wrong to associate any
                // following error with this property.
                iter.parser.last_parsed_property_id = None;
            },
            Err((error, slice)) => {
                iter.parser.declarations.clear();

                if context.error_reporting_enabled() {
                    let property = iter.parser.last_parsed_property_id.take();
                    errors.push((error, slice, property));
                }
            },
        }
    }

    if !errors.is_empty() {
        report_css_errors(context, &block, selectors, &mut errors)
    }

    block
}
