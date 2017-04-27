/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A centralized set of stylesheets for a document.

use arc_ptr_eq;
use std::sync::Arc;
use stylesheets::Stylesheet;

/// The set of stylesheets effective for a given document.
pub struct StylesheetSet {
    /// The actual list of all the stylesheets that apply to the given document.
    ///
    /// This is only a list of top-level stylesheets, and as such it doesn't
    /// include recursive `@import` rules.
    stylesheets: Vec<Arc<Stylesheet>>,

    /// Whether the stylesheets list above has changed since the last restyle.
    dirty: bool,

    /// Has author style been disabled?
    author_style_disabled: bool,
}

impl StylesheetSet {
    /// Create a new empty StylesheetSet.
    pub fn new() -> Self {
        StylesheetSet {
            stylesheets: vec![],
            dirty: false,
            author_style_disabled: false,
        }
    }

    /// Returns whether author styles have been disabled for the current
    /// stylesheet set.
    pub fn author_style_disabled(&self) -> bool {
        self.author_style_disabled
    }

    fn remove_stylesheet_if_present(&mut self, sheet: &Arc<Stylesheet>) {
        self.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
    }

    /// Appends a new stylesheet to the current set.
    pub fn append_stylesheet(&mut self, sheet: &Arc<Stylesheet>) {
        self.remove_stylesheet_if_present(sheet);
        self.stylesheets.push(sheet.clone());
        self.dirty = true;
    }

    /// Prepend a new stylesheet to the current set.
    pub fn prepend_stylesheet(&mut self, sheet: &Arc<Stylesheet>) {
        self.remove_stylesheet_if_present(sheet);
        self.stylesheets.insert(0, sheet.clone());
        self.dirty = true;
    }

    /// Insert a given stylesheet before another stylesheet in the document.
    pub fn insert_stylesheet_before(&mut self,
                                    sheet: &Arc<Stylesheet>,
                                    before: &Arc<Stylesheet>) {
        self.remove_stylesheet_if_present(sheet);
        let index = self.stylesheets.iter().position(|x| {
            arc_ptr_eq(x, before)
        }).expect("`before` stylesheet not found");
        self.stylesheets.insert(index, sheet.clone());
        self.dirty = true;
    }

    /// Remove a given stylesheet from the set.
    pub fn remove_stylesheet(&mut self, sheet: &Arc<Stylesheet>) {
        self.remove_stylesheet_if_present(sheet);
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
    pub fn flush(&mut self) -> &[Arc<Stylesheet>] {
        self.dirty = false;
        &self.stylesheets
    }

    /// Mark the stylesheets as dirty, because something external may have
    /// invalidated it.
    ///
    /// FIXME(emilio): Make this more granular.
    pub fn force_dirty(&mut self) {
        self.dirty = true;
    }
}
