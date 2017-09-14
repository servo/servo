/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A centralized set of stylesheets for a document.

use dom::TElement;
use invalidation::stylesheets::StylesheetInvalidationSet;
use media_queries::Device;
use shared_lock::SharedRwLockReadGuard;
use std::slice;
use stylesheets::{Origin, OriginSet, OriginSetIterator, PerOrigin, StylesheetInDocument};

/// Entry for a StylesheetSet.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct StylesheetSetEntry<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    sheet: S,
    dirty: bool,
}

impl<S> StylesheetSetEntry<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    fn new(sheet: S) -> Self {
        Self { sheet, dirty: true }
    }
}

/// A iterator over the stylesheets of a list of entries in the StylesheetSet.
#[derive(Clone)]
pub struct StylesheetCollectionIterator<'a, S>(slice::Iter<'a, StylesheetSetEntry<S>>)
where
    S: StylesheetInDocument + PartialEq + 'static;

impl<'a, S> Iterator for StylesheetCollectionIterator<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    type Item = &'a S;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|entry| &entry.sheet)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// An iterator over the flattened view of the stylesheet collections.
#[derive(Clone)]
pub struct StylesheetIterator<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    origins: OriginSetIterator,
    collections: &'a PerOrigin<SheetCollection<S>>,
    current: Option<StylesheetCollectionIterator<'a, S>>,
}

impl<'a, S> Iterator for StylesheetIterator<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    type Item = &'a S;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current.is_none() {
                let next_origin = match self.origins.next() {
                    Some(o) => o,
                    None => return None,
                };

                self.current =
                    Some(self.collections.borrow_for_origin(&next_origin).iter());
            }

            if let Some(s) = self.current.as_mut().unwrap().next() {
                return Some(s)
            }

            self.current = None;
        }
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

/// A struct to iterate over the different stylesheets to be flushed.
pub struct StylesheetFlusher<'a, 'b, S>
where
    'b: 'a,
    S: StylesheetInDocument + PartialEq + 'static,
{
    guard: &'a SharedRwLockReadGuard<'b>,
    origins_dirty: OriginSetIterator,
    // NB: Bound to the StylesheetSet lifetime when constructed, see
    // StylesheetSet::flush.
    collections: *mut PerOrigin<SheetCollection<S>>,
    current: Option<slice::IterMut<'a, StylesheetSetEntry<S>>>,
    origin_data_validity: PerOrigin<OriginValidity>,
    author_style_disabled: bool,
    had_invalidations: bool,
}

/// The type of rebuild that we need to do for a given stylesheet.
pub enum SheetRebuildKind {
    /// A full rebuild, of both cascade data and invalidation data.
    Full,
    /// A partial rebuild, of only the cascade data.
    CascadeOnly,
}

impl SheetRebuildKind {
    /// Whether the stylesheet invalidation data should be rebuilt.
    pub fn should_rebuild_invalidation(&self) -> bool {
        matches!(*self, SheetRebuildKind::Full)
    }
}

impl<'a, 'b, S> StylesheetFlusher<'a, 'b, S>
where
    'b: 'a,
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// The data validity for a given origin.
    pub fn origin_validity(&self, origin: Origin) -> OriginValidity {
        *self.origin_data_validity.borrow_for_origin(&origin)
    }

    /// Returns whether running the whole flushing process would be a no-op.
    pub fn nothing_to_do(&self) -> bool {
        self.origins_dirty.clone().next().is_none()
    }

    /// Returns whether any DOM invalidations were processed as a result of the
    /// stylesheet flush.
    pub fn had_invalidations(&self) -> bool {
        self.had_invalidations
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
            self.origins_dirty.next().is_none(),
            "You're supposed to fully consume the flusher"
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
        use std::mem;

        loop {
            if self.current.is_none() {
                let next_origin = match self.origins_dirty.next() {
                    Some(o) => o,
                    None => return None,
                };

                // Should've been cleared already.
                debug_assert_eq!(
                    unsafe { &*self.collections }
                        .borrow_for_origin(&next_origin)
                        .data_validity,
                    OriginValidity::Valid
                );

                self.current =
                    Some(unsafe { &mut *self.collections }.borrow_mut_for_origin(&next_origin).entries.iter_mut());
            }

            let potential_sheet = match self.current.as_mut().unwrap().next() {
                Some(s) => s,
                None => {
                    self.current = None;
                    continue;
                }
            };

            let dirty = mem::replace(&mut potential_sheet.dirty, false);

            if dirty {
                // If the sheet was dirty, we need to do a full rebuild anyway.
                return Some((&potential_sheet.sheet, SheetRebuildKind::Full))
            }

            let origin = potential_sheet.sheet.contents(self.guard).origin;
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

#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct SheetCollection<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// The actual list of stylesheets.
    ///
    /// This is only a list of top-level stylesheets, and as such it doesn't
    /// include recursive `@import` rules.
    entries: Vec<StylesheetSetEntry<S>>,

    /// The validity of the data that was already there for a given origin.
    ///
    /// Note that an origin may appear on `origins_dirty`, but still have
    /// `OriginValidity::Valid`, if only sheets have been appended into it (in
    /// which case the existing data is valid, but the origin needs to be
    /// rebuilt).
    data_validity: OriginValidity,
}

impl<S> Default for SheetCollection<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    fn default() -> Self {
        Self {
            entries: vec![],
            data_validity: OriginValidity::Valid,
        }
    }
}

impl<S> SheetCollection<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// Returns the number of stylesheets in the set.
    fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns the `index`th stylesheet in the set if present.
    fn get(&self, index: usize) -> Option<&S> {
        self.entries.get(index).map(|e| &e.sheet)
    }

    fn remove(&mut self, sheet: &S) {
        let old_len = self.entries.len();
        self.entries.retain(|entry| entry.sheet != *sheet);
        if cfg!(feature = "servo") {
            // FIXME(emilio): Make Gecko's PresShell::AddUserSheet not suck.
            //
            // Hopefully that's not necessary for correctness, just somewhat
            // overkill.
            debug_assert!(self.entries.len() != old_len, "Sheet not found?");
        }
        // Removing sheets makes us tear down the whole cascade and invalidation
        // data.
        self.set_data_validity_at_least(OriginValidity::FullyInvalid);
    }

    fn contains(&self, sheet: &S) -> bool {
        self.entries.iter().any(|e| e.sheet == *sheet)
    }

    /// Appends a given sheet into the collection.
    fn append(&mut self, sheet: S) {
        debug_assert!(!self.contains(&sheet));
        self.entries.push(StylesheetSetEntry::new(sheet))
        // Appending sheets doesn't alter the validity of the existing data, so
        // we don't need to change `data_validity` here.
    }

    fn insert_before(&mut self, sheet: S, before_sheet: &S) {
        debug_assert!(!self.contains(&sheet));

        let index = self.entries.iter().position(|entry| {
            entry.sheet == *before_sheet
        }).expect("`before_sheet` stylesheet not found");

        // Inserting stylesheets somewhere but at the end changes the validity
        // of the cascade data, but not the invalidation data.
        self.set_data_validity_at_least(OriginValidity::CascadeInvalid);
        self.entries.insert(index, StylesheetSetEntry::new(sheet));
    }

    fn set_data_validity_at_least(&mut self, validity: OriginValidity) {
        use std::cmp;
        self.data_validity = cmp::max(validity, self.data_validity);
    }

    fn prepend(&mut self, sheet: S) {
        debug_assert!(!self.contains(&sheet));
        // Inserting stylesheets somewhere but at the end changes the validity
        // of the cascade data, but not the invalidation data.
        self.set_data_validity_at_least(OriginValidity::CascadeInvalid);
        self.entries.insert(0, StylesheetSetEntry::new(sheet));
    }

    /// Returns an iterator over the current list of stylesheets.
    fn iter(&self) -> StylesheetCollectionIterator<S> {
        StylesheetCollectionIterator(self.entries.iter())
    }
}

/// The set of stylesheets effective for a given document.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct StylesheetSet<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// The collections of sheets per each origin.
    collections: PerOrigin<SheetCollection<S>>,

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
            collections: Default::default(),
            invalidations: StylesheetInvalidationSet::new(),
            origins_dirty: OriginSet::empty(),
            author_style_disabled: false,
        }
    }

    /// Returns the number of stylesheets in the set.
    pub fn len(&self) -> usize {
        self.collections.iter_origins().fold(0, |s, (item, _)| s + item.len())
    }

    /// Returns the `index`th stylesheet in the set for the given origin.
    pub fn get(&self, origin: Origin, index: usize) -> Option<&S> {
        self.collections.borrow_for_origin(&origin).get(index)
    }

    /// Returns whether author styles have been disabled for the current
    /// stylesheet set.
    pub fn author_style_disabled(&self) -> bool {
        self.author_style_disabled
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
        self.collect_invalidations_for(device, &sheet, guard);
        let origin = sheet.contents(guard).origin;
        self.collections.borrow_mut_for_origin(&origin).append(sheet);
    }

    /// Prepend a new stylesheet to the current set.
    pub fn prepend_stylesheet(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        guard: &SharedRwLockReadGuard
    ) {
        debug!("StylesheetSet::prepend_stylesheet");
        self.collect_invalidations_for(device, &sheet, guard);

        let origin = sheet.contents(guard).origin;
        self.collections.borrow_mut_for_origin(&origin).prepend(sheet)
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
        self.collect_invalidations_for(device, &sheet, guard);

        let origin = sheet.contents(guard).origin;
        self.collections
            .borrow_mut_for_origin(&origin)
            .insert_before(sheet, &before_sheet)
    }

    /// Remove a given stylesheet from the set.
    pub fn remove_stylesheet(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        guard: &SharedRwLockReadGuard,
    ) {
        debug!("StylesheetSet::remove_stylesheet");
        self.collect_invalidations_for(device, &sheet, guard);

        let origin = sheet.contents(guard).origin;
        self.collections.borrow_mut_for_origin(&origin).remove(&sheet)
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

    /// Flush the current set, unmarking it as dirty, and returns a
    /// `StylesheetFlusher` in order to rebuild the stylist.
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

        let had_invalidations = self.invalidations.flush(document_element);
        let origins_dirty =
            mem::replace(&mut self.origins_dirty, OriginSet::empty()).iter();

        let mut origin_data_validity = PerOrigin::<OriginValidity>::default();
        for origin in origins_dirty.clone() {
            let collection = self.collections.borrow_mut_for_origin(&origin);
            *origin_data_validity.borrow_mut_for_origin(&origin) =
                mem::replace(&mut collection.data_validity, OriginValidity::Valid);
        }

        StylesheetFlusher {
            collections: &mut self.collections,
            author_style_disabled: self.author_style_disabled,
            had_invalidations,
            origins_dirty,
            origin_data_validity,
            guard,
            current: None,
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

    /// Return an iterator over the flattened view of all the stylesheets.
    pub fn iter(&self) -> StylesheetIterator<S> {
        StylesheetIterator {
            origins: OriginSet::all().iter(),
            collections: &self.collections,
            current: None,
        }
    }

    /// Mark the stylesheets for the specified origin as dirty, because
    /// something external may have invalidated it.
    pub fn force_dirty(&mut self, origins: OriginSet) {
        self.invalidations.invalidate_fully();
        self.origins_dirty |= origins;
        for origin in origins.iter() {
            // We don't know what happened, assume the worse.
            self.collections
                .borrow_mut_for_origin(&origin)
                .set_data_validity_at_least(OriginValidity::FullyInvalid);
        }
    }
}
