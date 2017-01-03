/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed to style a Gecko document.

use animation::Animation;
use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use dom::OpaqueNode;
use euclid::size::TypedSize2D;
use gecko_bindings::bindings::RawServoStyleSet;
use gecko_bindings::sugar::ownership::{HasBoxFFI, HasFFI, HasSimpleFFI};
use media_queries::{Device, MediaType};
use num_cpus;
use parking_lot::RwLock;
use rayon;
use std::cmp;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use style_traits::ViewportPx;
use stylesheets::Stylesheet;
use stylist::Stylist;

/// The container for data that a Servo-backed Gecko document needs to style
/// itself.
pub struct PerDocumentStyleDataImpl {
    /// Rule processor.
    pub stylist: Arc<Stylist>,

    /// List of stylesheets, mirrored from Gecko.
    pub stylesheets: Vec<Arc<Stylesheet>>,

    /// Whether the stylesheets list above has changed since the last restyle.
    pub stylesheets_changed: bool,

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

    /// The worker thread pool.
    /// FIXME(bholley): This shouldn't be per-document.
    pub work_queue: Option<rayon::ThreadPool>,

    /// The number of threads of the work queue.
    pub num_threads: usize,
}

/// The data itself is an `AtomicRefCell`, which guarantees the proper semantics
/// and unexpected races while trying to mutate it.
pub struct PerDocumentStyleData(AtomicRefCell<PerDocumentStyleDataImpl>);

lazy_static! {
    /// The number of layout threads, computed statically.
    pub static ref NUM_THREADS: usize = {
        match env::var("STYLO_THREADS").map(|s| s.parse::<usize>().expect("invalid STYLO_THREADS")) {
            Ok(num) => num,
            _ => cmp::max(num_cpus::get() * 3 / 4, 1),
        }
    };
}

impl PerDocumentStyleData {
    /// Create a dummy `PerDocumentStyleData`.
    pub fn new() -> Self {
        // FIXME(bholley): Real window size.
        let window_size: TypedSize2D<f32, ViewportPx> = TypedSize2D::new(800.0, 600.0);
        let device = Device::new(MediaType::Screen, window_size);

        let (new_anims_sender, new_anims_receiver) = channel();

        PerDocumentStyleData(AtomicRefCell::new(PerDocumentStyleDataImpl {
            stylist: Arc::new(Stylist::new(device)),
            stylesheets: vec![],
            stylesheets_changed: true,
            new_animations_sender: new_anims_sender,
            new_animations_receiver: new_anims_receiver,
            running_animations: Arc::new(RwLock::new(HashMap::new())),
            expired_animations: Arc::new(RwLock::new(HashMap::new())),
            work_queue: if *NUM_THREADS <= 1 {
                None
            } else {
                let configuration =
                    rayon::Configuration::new().set_num_threads(*NUM_THREADS);
                rayon::ThreadPool::new(configuration).ok()
            },
            num_threads: *NUM_THREADS,
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
    /// Recreate the style data if the stylesheets have changed.
    pub fn flush_stylesheets(&mut self) {
        // The stylist wants to be flushed if either the stylesheets change or the
        // device dimensions change. When we add support for media queries, we'll
        // need to detect the latter case and trigger a flush as well.
        if self.stylesheets_changed {
            let _ = Arc::get_mut(&mut self.stylist).unwrap()
                                                   .update(&self.stylesheets, None, true);
            self.stylesheets_changed = false;
        }
    }
}

unsafe impl HasFFI for PerDocumentStyleData {
    type FFIType = RawServoStyleSet;
}
unsafe impl HasSimpleFFI for PerDocumentStyleData {}
unsafe impl HasBoxFFI for PerDocumentStyleData {}

impl Drop for PerDocumentStyleDataImpl {
    fn drop(&mut self) {
        let _ = self.work_queue.take();
    }
}
