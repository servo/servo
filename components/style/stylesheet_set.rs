/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A centralized set of stylesheets for a document.

use stylearc::Arc;
use stylesheets::Stylesheet;

/// Entry for a StylesheetSet. We don't bother creating a constructor, because
/// there's no sensible defaults for the member variables.
pub struct StylesheetSetEntry {
    unique_id: u32,
    sheet: Arc<Stylesheet>,
}

/// The set of stylesheets effective for a given document.
pub struct StylesheetSet {
    /// The actual list of all the stylesheets that apply to the given document,
    /// each stylesheet associated with a unique ID.
    ///
    /// This is only a list of top-level stylesheets, and as such it doesn't
    /// include recursive `@import` rules.
    entries: Vec<StylesheetSetEntry>,

    /// Whether the entries list above has changed since the last restyle.
    dirty: bool,

    /// Has author style been disabled?
    author_style_disabled: bool,
}

impl StylesheetSet {
    /// Create a new empty StylesheetSet.
    pub fn new() -> Self {
        StylesheetSet {
            entries: vec![],
            dirty: false,
            author_style_disabled: false,
        }
    }

    /// Returns whether author styles have been disabled for the current
    /// stylesheet set.
    pub fn author_style_disabled(&self) -> bool {
        self.author_style_disabled
    }

    fn remove_stylesheet_if_present(&mut self, unique_id: u32) {
        self.entries.retain(|x| x.unique_id != unique_id);
    }

    /// Appends a new stylesheet to the current set.
    pub fn append_stylesheet(&mut self, sheet: &Arc<Stylesheet>,
                             unique_id: u32) {
        self.remove_stylesheet_if_present(unique_id);
        self.entries.push(StylesheetSetEntry {
            unique_id: unique_id,
            sheet: sheet.clone(),
        });
        self.dirty = true;
    }

    /// Prepend a new stylesheet to the current set.
    pub fn prepend_stylesheet(&mut self, sheet: &Arc<Stylesheet>,
                              unique_id: u32) {
        self.remove_stylesheet_if_present(unique_id);
        self.entries.insert(0, StylesheetSetEntry {
            unique_id: unique_id,
            sheet: sheet.clone(),
        });
        self.dirty = true;
    }

    /// Insert a given stylesheet before another stylesheet in the document.
    pub fn insert_stylesheet_before(&mut self,
                                    sheet: &Arc<Stylesheet>,
                                    unique_id: u32,
                                    before_unique_id: u32) {
        self.remove_stylesheet_if_present(unique_id);
        let index = self.entries.iter().position(|x| {
            x.unique_id == before_unique_id
        }).expect("`before_unique_id` stylesheet not found");
        self.entries.insert(index, StylesheetSetEntry {
            unique_id: unique_id,
            sheet: sheet.clone(),
        });
        self.dirty = true;
    }

    /// Remove a given stylesheet from the set.
    pub fn remove_stylesheet(&mut self, unique_id: u32) {
        self.remove_stylesheet_if_present(unique_id);
        self.dirty = true;
    }

    /// Notes that the author style has been disabled for this document.
    pub fn set_author_style_disabled(&mut self, disabled: bool) {
        if self.author_style_disabled == disabled {
            return;
        }
        self.author_style_disabled = disabled;
        self.dirty = true;
    }

    /// Returns whether the given set has changed from the last flush.
    pub fn has_changed(&self) -> bool {
        self.dirty
    }

    /// Flush the current set, unmarking it as dirty.
    pub fn flush(&mut self, sheets: &mut Vec<Arc<Stylesheet>>) {
        self.dirty = false;
        for entry in &self.entries {
            sheets.push(entry.sheet.clone())
        }
    }

    /// Mark the stylesheets as dirty, because something external may have
    /// invalidated it.
    ///
    /// FIXME(emilio): Make this more granular.
    pub fn force_dirty(&mut self) {
        self.dirty = true;
    }
}
