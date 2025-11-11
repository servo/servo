/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::media_queries::Device;
use style::shared_lock::SharedRwLockReadGuard;
use style::stylesheet_set::{AuthorStylesheetSet, DocumentStylesheetSet};
use style::stylesheets::{CustomMediaMap, StylesheetInDocument};

/// Functionality common to DocumentStylesheetSet and AuthorStylesheetSet.
pub(crate) enum StylesheetSetRef<'a, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// Author stylesheet set.
    Author(&'a mut AuthorStylesheetSet<S>),
    /// Document stylesheet set.
    Document(&'a mut DocumentStylesheetSet<S>),
}

impl<S> StylesheetSetRef<'_, S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// Appends a new stylesheet to the current set.
    ///
    /// No device implies not computing invalidations.
    pub(crate) fn append_stylesheet(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        guard: &SharedRwLockReadGuard,
    ) {
        let custom_media = &CustomMediaMap::default();
        match self {
            StylesheetSetRef::Author(set) => {
                set.append_stylesheet(device, custom_media, sheet, guard)
            },
            StylesheetSetRef::Document(set) => {
                set.append_stylesheet(device, custom_media, sheet, guard)
            },
        }
    }

    /// Insert a given stylesheet before another stylesheet in the document.
    pub(crate) fn insert_stylesheet_before(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        before_sheet: S,
        guard: &SharedRwLockReadGuard,
    ) {
        let custom_media = &CustomMediaMap::default();
        match self {
            StylesheetSetRef::Author(set) => {
                set.insert_stylesheet_before(device, custom_media, sheet, before_sheet, guard)
            },
            StylesheetSetRef::Document(set) => {
                set.insert_stylesheet_before(device, custom_media, sheet, before_sheet, guard)
            },
        }
    }

    /// Remove a given stylesheet from the set.
    pub(crate) fn remove_stylesheet(
        &mut self,
        device: Option<&Device>,
        sheet: S,
        guard: &SharedRwLockReadGuard,
    ) {
        let custom_media = &CustomMediaMap::default();
        match self {
            StylesheetSetRef::Author(set) => {
                set.remove_stylesheet(device, custom_media, sheet, guard)
            },
            StylesheetSetRef::Document(set) => {
                set.remove_stylesheet(device, custom_media, sheet, guard)
            },
        }
    }
}
