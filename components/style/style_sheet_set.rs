/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A centralized set of style_sheets for a document.

use dom::TElement;
use invalidation::style_sheets::StyleSheetInvalidationSet;
use media_queries::Device;
use shared_lock::SharedRwLockReadGuard;
use std::slice;
use style_sheets::{Origin, OriginSet, PerOrigin, StyleSheetInDocument};

/// Entry for a StyleSheetSet.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct StyleSheetSetEntry<S>
where
    S: StyleSheetInDocument + PartialEq + 'static,
{
    sheet: S,
    dirty: bool,
}

impl<S> StyleSheetSetEntry<S>
where
    S: StyleSheetInDocument + PartialEq + 'static,
{
    fn new(sheet: S) -> Self {
        Self { sheet, dirty: true }
    }
}

/// A iterator over the style_sheets of a list of entries in the StyleSheetSet.
#[derive(Clone)]
pub struct StyleSheetIterator<'a, S>(slice::Iter<'a, StyleSheetSetEntry<S>>)
where
    S: StyleSheetInDocument + PartialEq + 'static;

impl<'a, S> Iterator for StyleSheetIterator<'a, S>
where
    S: StyleSheetInDocument + PartialEq + 'static,
{
    type Item = &'a S;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|entry| &entry.sheet)
    }
}

/// The validity of the data in a given cascade origin.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum OriginValidity {
    /// The origin is clean, all the data already there is valid, though we may
    /// have new sheets at the end.
    Valid = 0,

    /// The cascade data is invalid, but not the invalidation data (which is
    /// order-independent), and thus only the cascade data should be inserted.
    CascadeInvalid = 1,

    /// Everything needs to be rebuilt.
    FullyInvalid = 2,
}

impl Default for OriginValidity {
    fn default() -> Self {
        OriginValidity::Valid
    }
}

/// A struct to iterate over the different style_sheets to be flushed.
pub struct StyleSheetFlusher<'a, 'b, S>
where
    'b: 'a,
    S: StyleSheetInDocument + PartialEq + 'static,
{
    iter: slice::IterMut<'a, StyleSheetSetEntry<S>>,
    guard: &'a SharedRwLockReadGuard<'b>,
    origins_dirty: OriginSet,
    origin_data_validity: PerOrigin<OriginValidity>,
    author_style_disabled: bool,
    had_invalidations: bool,
}

/// The type of rebuild that we need to do for a given style_sheet.
pub enum SheetRebuildKind {
    /// A full rebuild, of both cascade data and invalidation data.
    Full,
    /// A partial rebuild, of only the cascade data.
    CascadeOnly,
}

impl SheetRebuildKind {
    /// Whether the style_sheet invalidation data should be rebuilt.
    pub fn should_rebuild_invalidation(&self) -> bool {
        matches!(*self, SheetRebuildKind::Full)
    }
}

impl<'a, 'b, S> StyleSheetFlusher<'a, 'b, S>
where
    'b: 'a,
    S: StyleSheetInDocument + PartialEq + 'static,
{
    /// The data validity for a given origin.
    pub fn origin_validity(&self, origin: Origin) -> OriginValidity {
        *self.origin_data_validity.borrow_for_origin(&origin)
    }

    /// Returns whether running the whole flushing process would be a no-op.
    pub fn nothing_to_do(&self) -> bool {
        self.origins_dirty.is_empty()
    }

    /// Returns whether any DOM invalidations were processed as a result of the
    /// style_sheet flush.
    pub fn had_invalidations(&self) -> bool {
        self.had_invalidations
    }
}

#[cfg(debug_assertions)]
impl<'a, 'b, S> Drop for StyleSheetFlusher<'a, 'b, S>
where
    'b: 'a,
    S: StyleSheetInDocument + PartialEq + 'static,
{
    fn drop(&mut self) {
        debug_assert!(
            self.iter.next().is_none(),
            "You're supposed to fully consume the flusher"
        );
    }
}

impl<'a, 'b, S> Iterator for StyleSheetFlusher<'a, 'b, S>
where
    'b: 'a,
    S: StyleSheetInDocument + PartialEq + 'static,
{
    type Item = (&'a S, SheetRebuildKind);

    fn next(&mut self) -> Option<Self::Item> {
        use std::mem;

        loop {
            let potential_sheet = match self.iter.next() {
                None => return None,
                Some(s) => s,
            };

            let dirty = mem::replace(&mut potential_sheet.dirty, false);

            if dirty {
                // If the sheet was dirty, we need to do a full rebuild anyway.
                return Some((&potential_sheet.sheet, SheetRebuildKind::Full))
            }

            let origin = potential_sheet.sheet.contents(self.guard).origin;
            if !self.origins_dirty.contains(origin.into()) {
                continue;
            }

            if self.author_style_disabled && matches!(origin, Origin::Author) {
                continue;
            }

            let rebuild_kind = match self.origin_validity(origin) {
                OriginValidity::Valid => continue,
                OriginValidity::CascadeInvalid => SheetRebuildKind::CascadeOnly,
                OriginValidity::FullyInvalid => SheetRebuildKind::Full,
            };

            return Some((&potential_sheet.sheet, rebuild_kind));
        }
    }
}

/// The set of style_sheets effective for a given document.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct StyleSheetSet<S>
where
    S: StyleSheetInDocument + PartialEq + 'static,
{
    /// The actual list of all the style_sheets that apply to the given document,
    /// each style_sheet associated with a unique ID.
    ///
    /// This is only a list of top-level style_sheets, and as such it doesn't
    /// include recursive `@import` rules.
    entries: Vec<StyleSheetSetEntry<S>>,

    /// The invalidations for style_sheets added or removed from this document.
    invalidations: StyleSheetInvalidationSet,

    /// The origins whose style_sheets have changed so far.
    origins_dirty: OriginSet,

    /// The validity of the data that was already there for a given origin.
    ///
    /// Note that an origin may appear on `origins_dirty`, but still have
    /// `OriginValidity::Valid`, if only sheets have been appended into it (in
    /// which case the existing data is valid, but the origin needs to be
    /// rebuilt).
    origin_data_validity: PerOrigin<OriginValidity>,

    /// Has author style been disabled?
    author_style_disabled: bool,
}

impl<S> StyleSheetSet<S>
where
    S: StyleSheetInDocument + PartialEq + 'static,
{
    /// Create a new empty StyleSheetSet.
    pub fn new() -> Self {
        StyleSheetSet {
            entries: vec![],
            invalidations: StyleSheetInvalidationSet::new(),
            origins_dirty: OriginSet::empty(),
            origin_data_validity: Default::default(),
            author_style_disabled: false,
        }
    }

    /// Returns the number of style_sheets in the set.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns the number of style_sheets in the set.
    pub fn get(&self, index: usize) -> Option<&S> {
        self.entries.get(index).map(|s| &s.sheet)
    }

    /// Returns whether author styles have been disabled for the current
    /// style_sheet set.
    pub fn author_style_disabled(&self) -> bool {
        self.author_style_disabled
    }

    fn remove_style_sheet_if_present(&mut self, sheet: &S) {
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

    fn set_data_validity_at_least(
        &mut self,
        origin: Origin,
        validity: OriginValidity,
    ) {
        use std::cmp;

        debug_assert!(
            self.origins_dirty.contains(origin.into()),
            "data_validity should be a subset of origins_dirty"
        );

        let existing_validity =
            self.origin_data_validity.borrow_mut_for_origin(&origin);

        *existing_validity = cmp::max(*existing_validity, validity);
    }

    /// Appends a new style_sheet to the current set.
    ///
    /// No device implies not computing invalidations.
    pub fn append_style_sheet(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        guard: &SharedRwLockReadGuard
    ) {
        debug!("StyleSheetSet::append_style_sheet");
        self.remove_style_sheet_if_present(&sheet);
        self.collect_invalidations_for(device, &sheet, guard);
        // Appending sheets doesn't alter the validity of the existing data, so
        // we don't need to change `origin_data_validity` here.
        self.entries.push(StyleSheetSetEntry::new(sheet));
    }

    /// Prepend a new style_sheet to the current set.
    pub fn prepend_style_sheet(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        guard: &SharedRwLockReadGuard
    ) {
        debug!("StyleSheetSet::prepend_style_sheet");
        self.remove_style_sheet_if_present(&sheet);
        self.collect_invalidations_for(device, &sheet, guard);

        // Inserting style_sheets somewhere but at the end changes the validity
        // of the cascade data, but not the invalidation data.
        self.set_data_validity_at_least(sheet.contents(guard).origin, OriginValidity::CascadeInvalid);

        self.entries.insert(0, StyleSheetSetEntry::new(sheet));
    }

    /// Insert a given style_sheet before another style_sheet in the document.
    pub fn insert_style_sheet_before(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        before_sheet: S,
        guard: &SharedRwLockReadGuard,
    ) {
        debug!("StyleSheetSet::insert_style_sheet_before");
        self.remove_style_sheet_if_present(&sheet);
        let index = self.entries.iter().position(|entry| {
            entry.sheet == before_sheet
        }).expect("`before_sheet` style_sheet not found");
        self.collect_invalidations_for(device, &sheet, guard);

        // Inserting style_sheets somewhere but at the end changes the validity
        // of the cascade data, but not the invalidation data.
        self.set_data_validity_at_least(sheet.contents(guard).origin, OriginValidity::CascadeInvalid);
        self.entries.insert(index, StyleSheetSetEntry::new(sheet));
    }

    /// Remove a given style_sheet from the set.
    pub fn remove_style_sheet(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        guard: &SharedRwLockReadGuard,
    ) {
        debug!("StyleSheetSet::remove_style_sheet");
        self.remove_style_sheet_if_present(&sheet);

        self.collect_invalidations_for(device, &sheet, guard);

        // Removing sheets makes us tear down the whole cascade and invalidation
        // data.
        self.set_data_validity_at_least(sheet.contents(guard).origin, OriginValidity::FullyInvalid);
    }

    /// Notes that the author style has been disabled for this document.
    pub fn set_author_style_disabled(&mut self, disabled: bool) {
        debug!("StyleSheetSet::set_author_style_disabled");
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

    /// Flush the current set, unmarking it as dirty, and returns a
    /// `StyleSheetFlusher` in order to rebuild the stylist.
    pub fn flush<'a, 'b, E>(
        &'a mut self,
        document_element: Option<E>,
        guard: &'a SharedRwLockReadGuard<'b>,
    ) -> StyleSheetFlusher<'a, 'b, S>
    where
        E: TElement,
    {
        use std::mem;

        debug!("StyleSheetSet::flush");

        let had_invalidations = self.invalidations.flush(document_element);
        let origins_dirty = mem::replace(&mut self.origins_dirty, OriginSet::empty());
        let origin_data_validity =
            mem::replace(&mut self.origin_data_validity, Default::default());

        StyleSheetFlusher {
            iter: self.entries.iter_mut(),
            author_style_disabled: self.author_style_disabled,
            had_invalidations,
            origins_dirty,
            origin_data_validity,
            guard,
        }
    }

    /// Flush style_sheets, but without running any of the invalidation passes.
    #[cfg(feature = "servo")]
    pub fn flush_without_invalidation(&mut self) -> OriginSet {
        use std::mem;

        debug!("StyleSheetSet::flush_without_invalidation");

        self.invalidations.clear();
        mem::replace(&mut self.origins_dirty, OriginSet::empty())
    }

    /// Returns an iterator over the current list of style_sheets.
    pub fn iter(&self) -> StyleSheetIterator<S> {
        StyleSheetIterator(self.entries.iter())
    }

    /// Mark the style_sheets for the specified origin as dirty, because
    /// something external may have invalidated it.
    pub fn force_dirty(&mut self, origins: OriginSet) {
        self.invalidations.invalidate_fully();
        self.origins_dirty |= origins;
        for origin in origins.iter() {
            // We don't know what happened, assume the worse.
            self.set_data_validity_at_least(origin, OriginValidity::FullyInvalid);
        }
    }
}
