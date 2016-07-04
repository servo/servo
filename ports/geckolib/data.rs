/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::Size2D;
use euclid::size::TypedSize2D;
use gecko_bindings::bindings::RawServoStyleSet;
use num_cpus;
use selector_impl::{Animation, SharedStyleContext, Stylist, Stylesheet};
use std::cmp;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use style::dom::OpaqueNode;
use style::media_queries::{Device, MediaType};
use style::parallel::WorkQueueData;
use style::workqueue::WorkQueue;
use util::geometry::ViewportPx;
use util::thread_state;

pub struct PerDocumentStyleData {
    /// Rule processor.
    pub stylist: Arc<Stylist>,

    /// List of stylesheets, mirrored from Gecko.
    pub stylesheets: Vec<Arc<Stylesheet>>,

    /// Whether the stylesheets list above has changed since the last restyle.
    pub stylesheets_changed: bool,

    // FIXME(bholley): Hook these up to something.
    pub new_animations_sender: Sender<Animation>,
    pub new_animations_receiver: Receiver<Animation>,
    pub running_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,
    pub expired_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,

    // FIXME(bholley): This shouldn't be per-document.
    pub work_queue: WorkQueue<SharedStyleContext, WorkQueueData>,
}

impl PerDocumentStyleData {
    pub fn new() -> PerDocumentStyleData {
        // FIXME(bholley): Real window size.
        let window_size: TypedSize2D<ViewportPx, f32> = Size2D::typed(800.0, 600.0);
        let device = Device::new(MediaType::Screen, window_size);

        let (new_anims_sender, new_anims_receiver) = channel();
        let num_threads = cmp::max(num_cpus::get() * 3 / 4, 1);

        PerDocumentStyleData {
            stylist: Arc::new(Stylist::new(device)),
            stylesheets: vec![],
            stylesheets_changed: true,
            new_animations_sender: new_anims_sender,
            new_animations_receiver: new_anims_receiver,
            running_animations: Arc::new(RwLock::new(HashMap::new())),
            expired_animations: Arc::new(RwLock::new(HashMap::new())),
            work_queue: WorkQueue::new("StyleWorker", thread_state::LAYOUT, num_threads),
        }
    }

    pub fn borrow_mut_from_raw<'a>(data: *mut RawServoStyleSet) -> &'a mut Self {
        unsafe { &mut *(data as *mut PerDocumentStyleData) }
    }
}

impl Drop for PerDocumentStyleData {
    fn drop(&mut self) {
        self.work_queue.shutdown();
    }
}
