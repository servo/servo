/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed to style a Gecko document.

use Atom;
use animation::Animation;
use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use dom::OpaqueNode;
use gecko::rules::{CounterStyleRule, FontFaceRule};
use gecko_bindings::bindings::RawServoStyleSet;
use gecko_bindings::structs::RawGeckoPresContextOwned;
use gecko_bindings::structs::nsIDocument;
use gecko_bindings::sugar::ownership::{HasBoxFFI, HasFFI, HasSimpleFFI};
use media_queries::Device;
use parking_lot::RwLock;
use properties::ComputedValues;
use shared_lock::{Locked, StylesheetGuards, SharedRwLockReadGuard};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use stylearc::Arc;
use stylesheet_set::StylesheetSet;
use stylesheets::Origin;
use stylist::{ExtraStyleData, Stylist};

/// The container for data that a Servo-backed Gecko document needs to style
/// itself.
pub struct PerDocumentStyleDataImpl {
    /// Rule processor.
    pub stylist: Stylist,

    /// List of stylesheets, mirrored from Gecko.
    pub stylesheets: StylesheetSet,

    // FIXME(bholley): Hook these up to something.
    /// Unused. Will go away when we actually implement transitions and
    /// animations properly.
    pub new_animations_sender: Sender<Animation>,
    /// Unused. Will go away when we actually implement transitions and
    /// animations properly.
    pub new_animations_receiver: Receiver<Animation>,
    /// Unused. Will go away when we actually implement transitions and
    /// animations properly.
    pub running_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,
    /// Unused. Will go away when we actually implement transitions and
    /// animations properly.
    pub expired_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,

    /// List of effective font face rules.
    pub font_faces: Vec<(Arc<Locked<FontFaceRule>>, Origin)>,
    /// Map for effective counter style rules.
    pub counter_styles: HashMap<Atom, Arc<Locked<CounterStyleRule>>>,
}

/// The data itself is an `AtomicRefCell`, which guarantees the proper semantics
/// and unexpected races while trying to mutate it.
pub struct PerDocumentStyleData(AtomicRefCell<PerDocumentStyleDataImpl>);

impl PerDocumentStyleData {
    /// Create a dummy `PerDocumentStyleData`.
    pub fn new(pres_context: RawGeckoPresContextOwned) -> Self {
        let device = Device::new(pres_context);
        let quirks_mode = unsafe {
            (*(*device.pres_context).mDocument.raw::<nsIDocument>()).mCompatMode
        };

        let (new_anims_sender, new_anims_receiver) = channel();

        PerDocumentStyleData(AtomicRefCell::new(PerDocumentStyleDataImpl {
            stylist: Stylist::new(device, quirks_mode.into()),
            stylesheets: StylesheetSet::new(),
            new_animations_sender: new_anims_sender,
            new_animations_receiver: new_anims_receiver,
            running_animations: Arc::new(RwLock::new(HashMap::new())),
            expired_animations: Arc::new(RwLock::new(HashMap::new())),
            font_faces: vec![],
            counter_styles: HashMap::new(),
        }))
    }

    /// Get an immutable reference to this style data.
    pub fn borrow(&self) -> AtomicRef<PerDocumentStyleDataImpl> {
        self.0.borrow()
    }

    /// Get an mutable reference to this style data.
    pub fn borrow_mut(&self) -> AtomicRefMut<PerDocumentStyleDataImpl> {
        self.0.borrow_mut()
    }
}

impl PerDocumentStyleDataImpl {
    /// Reset the device state because it may have changed.
    ///
    /// Implies also a stylesheet flush.
    pub fn reset_device(&mut self, guard: &SharedRwLockReadGuard) {
        Arc::get_mut(&mut self.stylist.device).unwrap().reset();
        self.stylesheets.force_dirty();
        self.flush_stylesheets(guard);
    }

    /// Recreate the style data if the stylesheets have changed.
    pub fn flush_stylesheets(&mut self, guard: &SharedRwLockReadGuard) {
        if !self.stylesheets.has_changed() {
            return;
        }

        let mut extra_data = ExtraStyleData {
            font_faces: &mut self.font_faces,
            counter_styles: &mut self.counter_styles,
        };

        let author_style_disabled = self.stylesheets.author_style_disabled();
        self.stylist.clear();
        self.stylist.rebuild(self.stylesheets.flush(),
                             &StylesheetGuards::same(guard),
                             /* ua_sheets = */ None,
                             /* stylesheets_changed = */ true,
                             author_style_disabled,
                             &mut extra_data);
    }

    /// Get the default computed values for this document.
    pub fn default_computed_values(&self) -> &Arc<ComputedValues> {
        self.stylist.device.default_computed_values_arc()
    }

    /// Clear the stylist.  This will be a no-op if the stylist is
    /// already cleared; the stylist handles that.
    pub fn clear_stylist(&mut self) {
        self.stylist.clear();
    }
}

unsafe impl HasFFI for PerDocumentStyleData {
    type FFIType = RawServoStyleSet;
}
unsafe impl HasSimpleFFI for PerDocumentStyleData {}
unsafe impl HasBoxFFI for PerDocumentStyleData {}
