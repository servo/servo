/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A centralized set of stylesheets for a document.

use dom::TElement;
use invalidation::stylesheets::StylesheetInvalidationSet;
use media_queries::Device;
use shared_lock::SharedRwLockReadGuard;
use std::slice;
use stylesheets::{Origin, OriginSet, StylesheetInDocument};

/// Entry for a StylesheetSet. We don't bother creating a constructor, because
/// there's no sensible defaults for the member variables.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct StylesheetSetEntry<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    sheet: S,
}

impl<S> StylesheetSetEntry<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    fn new(sheet: S) -> Self {
        Self { sheet }
    }
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

/// A struct to iterate over the different stylesheets to be flushed.
pub struct StylesheetFlusher<'a, 'b, S>
where
    'b: 'a,
    S: StylesheetInDocument + PartialEq + 'static,
{
    iter: slice::IterMut<'a, StylesheetSetEntry<S>>,
    guard: &'a SharedRwLockReadGuard<'b>,
    origins_dirty: OriginSet,
    author_style_disabled: bool,
}

/// The type of rebuild that we need to do for a given stylesheet.
pub enum SheetRebuildKind {
    /// For now we only support full rebuilds, in the future we'll implement
    /// partial rebuilds.
    Full,
}

impl<'a, 'b, S> StylesheetFlusher<'a, 'b, S>
where
    'b: 'a,
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// The set of origins to fully rebuild, which need to be cleared
    /// beforehand.
    pub fn origins_to_fully_rebuild(&self) -> OriginSet {
        self.origins_dirty
    }

    /// Returns whether running the whole flushing process would be a no-op.
    pub fn nothing_to_do(&self) -> bool {
        self.origins_dirty.is_empty()
    }
}

#[cfg(debug_assertions)]
impl<'a, 'b, S> Drop for StylesheetFlusher<'a, 'b, S>
where
    'b: 'a,
    S: StylesheetInDocument + PartialEq + 'static,
{
    fn drop(&mut self) {
        debug_assert!(
            self.iter.next().is_none(),
            "You're supposed to fully consume the flusher",
        );
    }
}

impl<'a, 'b, S> Iterator for StylesheetFlusher<'a, 'b, S>
where
    'b: 'a,
    S: StylesheetInDocument + PartialEq + 'static,
{
    type Item = (&'a S, SheetRebuildKind);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let potential_sheet = match self.iter.next() {
                None => return None,
                Some(s) => s,
            };

            let origin = potential_sheet.sheet.contents(self.guard).origin;
            if !self.origins_dirty.contains(origin.into()) {
                continue;
            }

            if self.author_style_disabled && matches!(origin, Origin::Author) {
                continue;
            }

            return Some((&potential_sheet.sheet, SheetRebuildKind::Full))
        }
    }
}

/// The set of stylesheets effective for a given document.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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

    /// The invalidations for stylesheets added or removed from this document.
    invalidations: StylesheetInvalidationSet,

    /// The origins whose stylesheets have changed so far.
    origins_dirty: OriginSet,

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
            invalidations: StylesheetInvalidationSet::new(),
            origins_dirty: OriginSet::empty(),
            author_style_disabled: false,
        }
    }

    /// Returns the number of stylesheets in the set.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns the number of stylesheets in the set.
    pub fn get(&self, index: usize) -> Option<&S> {
        self.entries.get(index).map(|s| &s.sheet)
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
        device: Option<&Device>,
        sheet: &S,
        guard: &SharedRwLockReadGuard,
    ) {
        if let Some(device) = device {
            self.invalidations.collect_invalidations_for(device, sheet, guard);
        }
        self.origins_dirty |= sheet.contents(guard).origin;
    }

    /// Appends a new stylesheet to the current set.
    ///
    /// No device implies not computing invalidations.
    pub fn append_stylesheet(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        guard: &SharedRwLockReadGuard
    ) {
        debug!("StylesheetSet::append_stylesheet");
        self.remove_stylesheet_if_present(&sheet);
        self.collect_invalidations_for(device, &sheet, guard);
        self.entries.push(StylesheetSetEntry::new(sheet));
    }

    /// Prepend a new stylesheet to the current set.
    pub fn prepend_stylesheet(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        guard: &SharedRwLockReadGuard
    ) {
        debug!("StylesheetSet::prepend_stylesheet");
        self.remove_stylesheet_if_present(&sheet);
        self.collect_invalidations_for(device, &sheet, guard);
        self.entries.insert(0, StylesheetSetEntry::new(sheet));
    }

    /// Insert a given stylesheet before another stylesheet in the document.
    pub fn insert_stylesheet_before(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        before_sheet: S,
        guard: &SharedRwLockReadGuard,
    ) {
        debug!("StylesheetSet::insert_stylesheet_before");
        self.remove_stylesheet_if_present(&sheet);
        let index = self.entries.iter().position(|entry| {
            entry.sheet == before_sheet
        }).expect("`before_sheet` stylesheet not found");
        self.collect_invalidations_for(device, &sheet, guard);
        self.entries.insert(index, StylesheetSetEntry::new(sheet));
    }

    /// Remove a given stylesheet from the set.
    pub fn remove_stylesheet(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        guard: &SharedRwLockReadGuard,
    ) {
        debug!("StylesheetSet::remove_stylesheet");
        self.remove_stylesheet_if_present(&sheet);
        self.collect_invalidations_for(device, &sheet, guard);
    }

    /// Notes that the author style has been disabled for this document.
    pub fn set_author_style_disabled(&mut self, disabled: bool) {
        debug!("StylesheetSet::set_author_style_disabled");
        if self.author_style_disabled == disabled {
            return;
        }
        self.author_style_disabled = disabled;
        self.invalidations.invalidate_fully();
        self.origins_dirty |= Origin::Author;
    }

    /// Returns whether the given set has changed from the last flush.
    pub fn has_changed(&self) -> bool {
        !self.origins_dirty.is_empty()
    }

    /// Flush the current set, unmarking it as dirty, and returns the damaged
    /// origins.
    pub fn flush<'a, 'b, E>(
        &'a mut self,
        document_element: Option<E>,
        guard: &'a SharedRwLockReadGuard<'b>,
    ) -> StylesheetFlusher<'a, 'b, S>
    where
        E: TElement,
    {
        use std::mem;

        debug!("StylesheetSet::flush");

        self.invalidations.flush(document_element);
        let origins_dirty = mem::replace(&mut self.origins_dirty, OriginSet::empty());

        StylesheetFlusher {
            iter: self.entries.iter_mut(),
            author_style_disabled: self.author_style_disabled,
            origins_dirty,
            guard,
        }
    }

    /// Flush stylesheets, but without running any of the invalidation passes.
    #[cfg(feature = "servo")]
    pub fn flush_without_invalidation(&mut self) -> OriginSet {
        use std::mem;

        debug!("StylesheetSet::flush_without_invalidation");

        self.invalidations.clear();
        mem::replace(&mut self.origins_dirty, OriginSet::empty())
    }

    /// Returns an iterator over the current list of stylesheets.
    pub fn iter(&self) -> StylesheetIterator<S> {
        StylesheetIterator(self.entries.iter())
    }

    /// Mark the stylesheets for the specified origin as dirty, because
    /// something external may have invalidated it.
    pub fn force_dirty(&mut self, origins: OriginSet) {
        self.invalidations.invalidate_fully();
        self.origins_dirty |= origins;
    }
}
