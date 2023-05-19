/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A centralized set of stylesheets for a document.

use crate::dom::TElement;
use crate::invalidation::stylesheets::{RuleChangeKind, StylesheetInvalidationSet};
use crate::media_queries::Device;
use crate::selector_parser::SnapshotMap;
use crate::shared_lock::SharedRwLockReadGuard;
use crate::stylesheets::{
    CssRule, Origin, OriginSet, OriginSetIterator, PerOrigin, StylesheetInDocument,
};
use std::{mem, slice};

/// Entry for a StylesheetSet.
#[derive(MallocSizeOf)]
struct StylesheetSetEntry<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// The sheet.
    sheet: S,

    /// Whether this sheet has been part of at least one flush.
    committed: bool,
}

impl<S> StylesheetSetEntry<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    fn new(sheet: S) -> Self {
        Self {
            sheet,
            committed: false,
        }
    }
}

/// A iterator over the stylesheets of a list of entries in the StylesheetSet.
pub struct StylesheetCollectionIterator<'a, S>(slice::Iter<'a, StylesheetSetEntry<S>>)
where
    S: StylesheetInDocument + PartialEq + 'static;

impl<'a, S> Clone for StylesheetCollectionIterator<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    fn clone(&self) -> Self {
        StylesheetCollectionIterator(self.0.clone())
    }
}

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
    current: Option<(Origin, StylesheetCollectionIterator<'a, S>)>,
}

impl<'a, S> Iterator for StylesheetIterator<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    type Item = (&'a S, Origin);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current.is_none() {
                let next_origin = self.origins.next()?;

                self.current = Some((
                    next_origin,
                    self.collections.borrow_for_origin(&next_origin).iter(),
                ));
            }

            {
                let (origin, ref mut iter) = *self.current.as_mut().unwrap();
                if let Some(s) = iter.next() {
                    return Some((s, origin));
                }
            }

            self.current = None;
        }
    }
}

/// The validity of the data in a given cascade origin.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub enum DataValidity {
    /// The origin is clean, all the data already there is valid, though we may
    /// have new sheets at the end.
    Valid = 0,

    /// The cascade data is invalid, but not the invalidation data (which is
    /// order-independent), and thus only the cascade data should be inserted.
    CascadeInvalid = 1,

    /// Everything needs to be rebuilt.
    FullyInvalid = 2,
}

impl Default for DataValidity {
    fn default() -> Self {
        DataValidity::Valid
    }
}

/// A struct to iterate over the different stylesheets to be flushed.
pub struct DocumentStylesheetFlusher<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    collections: &'a mut PerOrigin<SheetCollection<S>>,
    had_invalidations: bool,
}

/// The type of rebuild that we need to do for a given stylesheet.
#[derive(Clone, Copy, Debug)]
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

impl<'a, S> DocumentStylesheetFlusher<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// Returns a flusher for `origin`.
    pub fn flush_origin(&mut self, origin: Origin) -> SheetCollectionFlusher<S> {
        self.collections.borrow_mut_for_origin(&origin).flush()
    }

    /// Returns the list of stylesheets for `origin`.
    ///
    /// Only used for UA sheets.
    pub fn origin_sheets(&mut self, origin: Origin) -> StylesheetCollectionIterator<S> {
        self.collections.borrow_mut_for_origin(&origin).iter()
    }

    /// Returns whether any DOM invalidations were processed as a result of the
    /// stylesheet flush.
    #[inline]
    pub fn had_invalidations(&self) -> bool {
        self.had_invalidations
    }
}

/// A flusher struct for a given collection, that takes care of returning the
/// appropriate stylesheets that need work.
pub struct SheetCollectionFlusher<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    // TODO: This can be made an iterator again once
    // https://github.com/rust-lang/rust/pull/82771 lands on stable.
    entries: &'a mut [StylesheetSetEntry<S>],
    validity: DataValidity,
    dirty: bool,
}

impl<'a, S> SheetCollectionFlusher<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// Whether the collection was originally dirty.
    #[inline]
    pub fn dirty(&self) -> bool {
        self.dirty
    }

    /// What the state of the sheet data is.
    #[inline]
    pub fn data_validity(&self) -> DataValidity {
        self.validity
    }

    /// Returns an iterator over the remaining list of sheets to consume.
    pub fn sheets<'b>(&'b self) -> impl Iterator<Item = &'b S> {
        self.entries.iter().map(|entry| &entry.sheet)
    }
}

impl<'a, S> SheetCollectionFlusher<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// Iterates over all sheets and values that we have to invalidate.
    ///
    /// TODO(emilio): This would be nicer as an iterator but we can't do that
    /// until https://github.com/rust-lang/rust/pull/82771 stabilizes.
    ///
    /// Since we don't have a good use-case for partial iteration, this does the
    /// trick for now.
    pub fn each(self, mut callback: impl FnMut(&S, SheetRebuildKind) -> bool) {
        for potential_sheet in self.entries.iter_mut() {
            let committed = mem::replace(&mut potential_sheet.committed, true);
            let rebuild_kind = if !committed {
                // If the sheet was uncommitted, we need to do a full rebuild
                // anyway.
                SheetRebuildKind::Full
            } else {
                match self.validity {
                    DataValidity::Valid => continue,
                    DataValidity::CascadeInvalid => SheetRebuildKind::CascadeOnly,
                    DataValidity::FullyInvalid => SheetRebuildKind::Full,
                }
            };

            if !callback(&potential_sheet.sheet, rebuild_kind) {
                return;
            }
        }
    }
}

#[derive(MallocSizeOf)]
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
    /// `DataValidity::Valid`, if only sheets have been appended into it (in
    /// which case the existing data is valid, but the origin needs to be
    /// rebuilt).
    data_validity: DataValidity,

    /// Whether anything in the collection has changed. Note that this is
    /// different from `data_validity`, in the sense that after a sheet append,
    /// the data validity is still `Valid`, but we need to be marked as dirty.
    dirty: bool,
}

impl<S> Default for SheetCollection<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    fn default() -> Self {
        Self {
            entries: vec![],
            data_validity: DataValidity::Valid,
            dirty: false,
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
        let index = self.entries.iter().position(|entry| entry.sheet == *sheet);
        if cfg!(feature = "gecko") && index.is_none() {
            // FIXME(emilio): Make Gecko's PresShell::AddUserSheet not suck.
            return;
        }
        let sheet = self.entries.remove(index.unwrap());
        // Removing sheets makes us tear down the whole cascade and invalidation
        // data, but only if the sheet has been involved in at least one flush.
        // Checking whether the sheet has been committed allows us to avoid
        // rebuilding the world when sites quickly append and remove a
        // stylesheet.
        //
        // See bug 1434756.
        if sheet.committed {
            self.set_data_validity_at_least(DataValidity::FullyInvalid);
        } else {
            self.dirty = true;
        }
    }

    fn contains(&self, sheet: &S) -> bool {
        self.entries.iter().any(|e| e.sheet == *sheet)
    }

    /// Appends a given sheet into the collection.
    fn append(&mut self, sheet: S) {
        debug_assert!(!self.contains(&sheet));
        self.entries.push(StylesheetSetEntry::new(sheet));
        // Appending sheets doesn't alter the validity of the existing data, so
        // we don't need to change `data_validity` here.
        //
        // But we need to be marked as dirty, otherwise we'll never add the new
        // sheet!
        self.dirty = true;
    }

    fn insert_before(&mut self, sheet: S, before_sheet: &S) {
        debug_assert!(!self.contains(&sheet));

        let index = self
            .entries
            .iter()
            .position(|entry| entry.sheet == *before_sheet)
            .expect("`before_sheet` stylesheet not found");

        // Inserting stylesheets somewhere but at the end changes the validity
        // of the cascade data, but not the invalidation data.
        self.set_data_validity_at_least(DataValidity::CascadeInvalid);
        self.entries.insert(index, StylesheetSetEntry::new(sheet));
    }

    fn set_data_validity_at_least(&mut self, validity: DataValidity) {
        use std::cmp;

        debug_assert_ne!(validity, DataValidity::Valid);

        self.dirty = true;
        self.data_validity = cmp::max(validity, self.data_validity);
    }

    /// Returns an iterator over the current list of stylesheets.
    fn iter(&self) -> StylesheetCollectionIterator<S> {
        StylesheetCollectionIterator(self.entries.iter())
    }

    fn flush(&mut self) -> SheetCollectionFlusher<S> {
        let dirty = mem::replace(&mut self.dirty, false);
        let validity = mem::replace(&mut self.data_validity, DataValidity::Valid);

        SheetCollectionFlusher {
            entries: &mut self.entries,
            dirty,
            validity,
        }
    }
}

/// The set of stylesheets effective for a given document.
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct DocumentStylesheetSet<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// The collections of sheets per each origin.
    collections: PerOrigin<SheetCollection<S>>,

    /// The invalidations for stylesheets added or removed from this document.
    invalidations: StylesheetInvalidationSet,
}

/// This macro defines methods common to DocumentStylesheetSet and
/// AuthorStylesheetSet.
///
/// We could simplify the setup moving invalidations to SheetCollection, but
/// that would imply not sharing invalidations across origins of the same
/// documents, which is slightly annoying.
macro_rules! sheet_set_methods {
    ($set_name:expr) => {
        fn collect_invalidations_for(
            &mut self,
            device: Option<&Device>,
            sheet: &S,
            guard: &SharedRwLockReadGuard,
        ) {
            if let Some(device) = device {
                self.invalidations
                    .collect_invalidations_for(device, sheet, guard);
            }
        }

        /// Appends a new stylesheet to the current set.
        ///
        /// No device implies not computing invalidations.
        pub fn append_stylesheet(
            &mut self,
            device: Option<&Device>,
            sheet: S,
            guard: &SharedRwLockReadGuard,
        ) {
            debug!(concat!($set_name, "::append_stylesheet"));
            self.collect_invalidations_for(device, &sheet, guard);
            let collection = self.collection_for(&sheet);
            collection.append(sheet);
        }

        /// Insert a given stylesheet before another stylesheet in the document.
        pub fn insert_stylesheet_before(
            &mut self,
            device: Option<&Device>,
            sheet: S,
            before_sheet: S,
            guard: &SharedRwLockReadGuard,
        ) {
            debug!(concat!($set_name, "::insert_stylesheet_before"));
            self.collect_invalidations_for(device, &sheet, guard);

            let collection = self.collection_for(&sheet);
            collection.insert_before(sheet, &before_sheet);
        }

        /// Remove a given stylesheet from the set.
        pub fn remove_stylesheet(
            &mut self,
            device: Option<&Device>,
            sheet: S,
            guard: &SharedRwLockReadGuard,
        ) {
            debug!(concat!($set_name, "::remove_stylesheet"));
            self.collect_invalidations_for(device, &sheet, guard);

            let collection = self.collection_for(&sheet);
            collection.remove(&sheet)
        }

        /// Notify the set that a rule from a given stylesheet has changed
        /// somehow.
        pub fn rule_changed(
            &mut self,
            device: Option<&Device>,
            sheet: &S,
            rule: &CssRule,
            guard: &SharedRwLockReadGuard,
            change_kind: RuleChangeKind,
        ) {
            if let Some(device) = device {
                let quirks_mode = device.quirks_mode();
                self.invalidations.rule_changed(
                    sheet,
                    rule,
                    guard,
                    device,
                    quirks_mode,
                    change_kind,
                );
            }

            let validity = match change_kind {
                // Insertion / Removals need to rebuild both the cascade and
                // invalidation data. For generic changes this is conservative,
                // could be optimized on a per-case basis.
                RuleChangeKind::Generic | RuleChangeKind::Insertion | RuleChangeKind::Removal => {
                    DataValidity::FullyInvalid
                },
                // TODO(emilio): This, in theory, doesn't need to invalidate
                // style data, if the rule we're modifying is actually in the
                // CascadeData already.
                //
                // But this is actually a bit tricky to prove, because when we
                // copy-on-write a stylesheet we don't bother doing a rebuild,
                // so we may still have rules from the original stylesheet
                // instead of the cloned one that we're modifying. So don't
                // bother for now and unconditionally rebuild, it's no worse
                // than what we were already doing anyway.
                //
                // Maybe we could record whether we saw a clone in this flush,
                // and if so do the conservative thing, otherwise just
                // early-return.
                RuleChangeKind::StyleRuleDeclarations => DataValidity::FullyInvalid,
            };

            let collection = self.collection_for(&sheet);
            collection.set_data_validity_at_least(validity);
        }
    };
}

impl<S> DocumentStylesheetSet<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// Create a new empty DocumentStylesheetSet.
    pub fn new() -> Self {
        Self {
            collections: Default::default(),
            invalidations: StylesheetInvalidationSet::new(),
        }
    }

    fn collection_for(&mut self, sheet: &S) -> &mut SheetCollection<S> {
        let origin = sheet.contents().origin;
        self.collections.borrow_mut_for_origin(&origin)
    }

    sheet_set_methods!("DocumentStylesheetSet");

    /// Returns the number of stylesheets in the set.
    pub fn len(&self) -> usize {
        self.collections
            .iter_origins()
            .fold(0, |s, (item, _)| s + item.len())
    }

    /// Returns the count of stylesheets for a given origin.
    #[inline]
    pub fn sheet_count(&self, origin: Origin) -> usize {
        self.collections.borrow_for_origin(&origin).len()
    }

    /// Returns the `index`th stylesheet in the set for the given origin.
    #[inline]
    pub fn get(&self, origin: Origin, index: usize) -> Option<&S> {
        self.collections.borrow_for_origin(&origin).get(index)
    }

    /// Returns whether the given set has changed from the last flush.
    pub fn has_changed(&self) -> bool {
        !self.invalidations.is_empty() ||
            self.collections
                .iter_origins()
                .any(|(collection, _)| collection.dirty)
    }

    /// Flush the current set, unmarking it as dirty, and returns a
    /// `DocumentStylesheetFlusher` in order to rebuild the stylist.
    pub fn flush<E>(
        &mut self,
        document_element: Option<E>,
        snapshots: Option<&SnapshotMap>,
    ) -> DocumentStylesheetFlusher<S>
    where
        E: TElement,
    {
        debug!("DocumentStylesheetSet::flush");

        let had_invalidations = self.invalidations.flush(document_element, snapshots);

        DocumentStylesheetFlusher {
            collections: &mut self.collections,
            had_invalidations,
        }
    }

    /// Flush stylesheets, but without running any of the invalidation passes.
    #[cfg(feature = "servo")]
    pub fn flush_without_invalidation(&mut self) -> OriginSet {
        debug!("DocumentStylesheetSet::flush_without_invalidation");

        let mut origins = OriginSet::empty();
        self.invalidations.clear();

        for (collection, origin) in self.collections.iter_mut_origins() {
            if collection.flush().dirty() {
                origins |= origin;
            }
        }

        origins
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
        for origin in origins.iter() {
            // We don't know what happened, assume the worse.
            self.collections
                .borrow_mut_for_origin(&origin)
                .set_data_validity_at_least(DataValidity::FullyInvalid);
        }
    }
}

/// The set of stylesheets effective for a given Shadow Root.
#[derive(MallocSizeOf)]
pub struct AuthorStylesheetSet<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// The actual style sheets.
    collection: SheetCollection<S>,
    /// The set of invalidations scheduled for this collection.
    invalidations: StylesheetInvalidationSet,
}

/// A struct to flush an author style sheet collection.
pub struct AuthorStylesheetFlusher<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// The actual flusher for the collection.
    pub sheets: SheetCollectionFlusher<'a, S>,
    /// Whether any sheet invalidation matched.
    pub had_invalidations: bool,
}

impl<S> AuthorStylesheetSet<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// Create a new empty AuthorStylesheetSet.
    #[inline]
    pub fn new() -> Self {
        Self {
            collection: Default::default(),
            invalidations: StylesheetInvalidationSet::new(),
        }
    }

    /// Whether anything has changed since the last time this was flushed.
    pub fn dirty(&self) -> bool {
        self.collection.dirty
    }

    /// Whether the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.collection.len() == 0
    }

    /// Returns the `index`th stylesheet in the collection of author styles if present.
    pub fn get(&self, index: usize) -> Option<&S> {
        self.collection.get(index)
    }

    /// Returns the number of author stylesheets.
    pub fn len(&self) -> usize {
        self.collection.len()
    }

    fn collection_for(&mut self, _sheet: &S) -> &mut SheetCollection<S> {
        &mut self.collection
    }

    sheet_set_methods!("AuthorStylesheetSet");

    /// Iterate over the list of stylesheets.
    pub fn iter(&self) -> StylesheetCollectionIterator<S> {
        self.collection.iter()
    }

    /// Mark the sheet set dirty, as appropriate.
    pub fn force_dirty(&mut self) {
        self.invalidations.invalidate_fully();
        self.collection
            .set_data_validity_at_least(DataValidity::FullyInvalid);
    }

    /// Flush the stylesheets for this author set.
    ///
    /// `host` is the root of the affected subtree, like the shadow host, for
    /// example.
    pub fn flush<E>(
        &mut self,
        host: Option<E>,
        snapshots: Option<&SnapshotMap>,
    ) -> AuthorStylesheetFlusher<S>
    where
        E: TElement,
    {
        let had_invalidations = self.invalidations.flush(host, snapshots);
        AuthorStylesheetFlusher {
            sheets: self.collection.flush(),
            had_invalidations,
        }
    }
}
