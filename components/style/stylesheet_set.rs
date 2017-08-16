/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A centralized set of stylesheets for a document.

use dom::TElement;
use invalidation::stylesheets::StylesheetInvalidationSet;
use shared_lock::SharedRwLockReadGuard;
use std::slice;
use stylesheets::{OriginSet, PerOrigin, StylesheetInDocument};
use stylist::Stylist;

/// Entry for a StylesheetSet. We don't bother creating a constructor, because
/// there's no sensible defaults for the member variables.
pub struct StylesheetSetEntry<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    sheet: S,
}

/// A iterator over the stylesheets of a list of entries in the StylesheetSet.
#[derive(Clone)]
pub struct StylesheetIterator<'a, S>(slice::Iter<'a, StylesheetSetEntry<S>>)
where
    S: StylesheetInDocument + PartialEq + 'static;

impl<'a, S> Iterator for StylesheetIterator<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    type Item = &'a S;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|entry| &entry.sheet)
    }
}

/// The set of stylesheets effective for a given document.
pub struct StylesheetSet<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// The actual list of all the stylesheets that apply to the given document,
    /// each stylesheet associated with a unique ID.
    ///
    /// This is only a list of top-level stylesheets, and as such it doesn't
    /// include recursive `@import` rules.
    entries: Vec<StylesheetSetEntry<S>>,

    /// Per-origin stylesheet invalidation data.
    invalidation_data: PerOrigin<InvalidationData>,

    /// Has author style been disabled?
    author_style_disabled: bool,
}

impl<S> StylesheetSet<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// Create a new empty StylesheetSet.
    pub fn new() -> Self {
        StylesheetSet {
            entries: vec![],
            invalidation_data: Default::default(),
            author_style_disabled: false,
        }
    }

    /// Returns whether author styles have been disabled for the current
    /// stylesheet set.
    pub fn author_style_disabled(&self) -> bool {
        self.author_style_disabled
    }

    fn remove_stylesheet_if_present(&mut self, sheet: &S) {
        self.entries.retain(|entry| entry.sheet != *sheet);
    }

    fn collect_invalidations_for(
        &mut self,
        stylist: &Stylist,
        sheet: &S,
        guard: &SharedRwLockReadGuard,
    ) {
        let origin = sheet.contents(guard).origin;
        let data = self.invalidation_data.borrow_mut_for_origin(&origin);
        data.invalidations.collect_invalidations_for(stylist, sheet, guard);
        data.dirty = true;
    }

    /// Appends a new stylesheet to the current set.
    pub fn append_stylesheet(
        &mut self,
        stylist: &Stylist,
        sheet: S,
        guard: &SharedRwLockReadGuard
    ) {
        debug!("StylesheetSet::append_stylesheet");
        self.remove_stylesheet_if_present(&sheet);
        self.collect_invalidations_for(stylist, &sheet, guard);
        self.entries.push(StylesheetSetEntry { sheet });
    }

    /// Prepend a new stylesheet to the current set.
    pub fn prepend_stylesheet(
        &mut self,
        stylist: &Stylist,
        sheet: S,
        guard: &SharedRwLockReadGuard
    ) {
        debug!("StylesheetSet::prepend_stylesheet");
        self.remove_stylesheet_if_present(&sheet);
        self.collect_invalidations_for(stylist, &sheet, guard);
        self.entries.insert(0, StylesheetSetEntry { sheet });
    }

    /// Insert a given stylesheet before another stylesheet in the document.
    pub fn insert_stylesheet_before(
        &mut self,
        stylist: &Stylist,
        sheet: S,
        before_sheet: S,
        guard: &SharedRwLockReadGuard
    ) {
        debug!("StylesheetSet::insert_stylesheet_before");
        self.remove_stylesheet_if_present(&sheet);
        let index = self.entries.iter().position(|entry| {
            entry.sheet == before_sheet
        }).expect("`before_sheet` stylesheet not found");
        self.collect_invalidations_for(stylist, &sheet, guard);
        self.entries.insert(index, StylesheetSetEntry { sheet });
    }

    /// Remove a given stylesheet from the set.
    pub fn remove_stylesheet(
        &mut self,
        stylist: &Stylist,
        sheet: S,
        guard: &SharedRwLockReadGuard,
    ) {
        debug!("StylesheetSet::remove_stylesheet");
        self.remove_stylesheet_if_present(&sheet);
        self.collect_invalidations_for(stylist, &sheet, guard);
    }

    /// Notes that the author style has been disabled for this document.
    pub fn set_author_style_disabled(&mut self, disabled: bool) {
        debug!("StylesheetSet::set_author_style_disabled");
        if self.author_style_disabled == disabled {
            return;
        }
        self.author_style_disabled = disabled;
        self.invalidation_data.author.invalidations.invalidate_fully();
        self.invalidation_data.author.dirty = true;
    }

    /// Returns whether the given set has changed from the last flush.
    pub fn has_changed(&self) -> bool {
        self.invalidation_data
            .iter_origins()
            .any(|(d, _)| d.dirty)
    }

    /// Flush the current set, unmarking it as dirty, and returns an iterator
    /// over the new stylesheet list.
    pub fn flush<E>(
        &mut self,
        document_element: Option<E>,
    ) -> (StylesheetIterator<S>, OriginSet)
    where
        E: TElement,
    {
        debug!("StylesheetSet::flush");
        debug_assert!(self.has_changed());

        let mut origins = OriginSet::empty();
        for (data, origin) in self.invalidation_data.iter_mut_origins() {
            if data.dirty {
                data.invalidations.flush(document_element);
                data.dirty = false;
                origins |= origin;
            }
        }

        (self.iter(), origins)
    }

    /// Returns an iterator over the current list of stylesheets.
    pub fn iter(&self) -> StylesheetIterator<S> {
        StylesheetIterator(self.entries.iter())
    }

    /// Mark the stylesheets for the specified origin as dirty, because
    /// something external may have invalidated it.
    pub fn force_dirty(&mut self, origins: OriginSet) {
        for origin in origins.iter() {
            let data = self.invalidation_data.borrow_mut_for_origin(&origin);
            data.invalidations.invalidate_fully();
            data.dirty = true;
        }
    }
}

struct InvalidationData {
    /// The stylesheet invalidations for this origin that we still haven't
    /// processed.
    invalidations: StylesheetInvalidationSet,

    /// Whether the sheets for this origin in the `StylesheetSet`'s entry list
    /// has changed since the last restyle.
    dirty: bool,
}

impl Default for InvalidationData {
    fn default() -> Self {
        InvalidationData {
            invalidations: StylesheetInvalidationSet::new(),
            dirty: false,
        }
    }
}
