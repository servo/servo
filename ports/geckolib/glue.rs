/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use app_units::Au;
use bindings::RawGeckoDocument;
use bindings::ServoNodeData;
use euclid::Size2D;
use euclid::size::TypedSize2D;
use num_cpus;
use std::cmp;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, RwLock};
use style::animation::Animation;
use style::context::{ReflowGoal, SharedStyleContext, StylistWrapper};
use style::dom::{TDocument, TNode};
use style::error_reporting::StdoutErrorReporter;
use style::media_queries::{Device, MediaType};
use style::parallel::{self, WorkQueueData};
use style::selector_matching::Stylist;
use style::stylesheets::Stylesheet;
use style::traversal::RecalcStyleOnly;
use util::geometry::ViewportPx;
use util::resource_files::set_resources_path;
use util::thread_state;
use util::workqueue::WorkQueue;
use wrapper::{GeckoDocument, GeckoNode, NonOpaqueStyleData};

/*
 * For Gecko->Servo function calls, we need to redeclare the same signature that was declared in
 * the C header in Gecko. In order to catch accidental mismatches, we run rust-bindgen against
 * those signatures as well, giving us a second declaration of all the Servo_* functions in this
 * crate. If there's a mismatch, LLVM will assert and abort, which is a rather awful thing to
 * depend on but good enough for our purposes.
 */

#[no_mangle]
pub extern "C" fn Servo_RestyleDocument(doc: *mut RawGeckoDocument) -> () {
    let document = unsafe { GeckoDocument::from_raw(doc) };
    let node = match document.root_node() {
        Some(x) => x,
        None => return,
    };

    // FIXME(bholley): Don't hardcode resources path. We may want to use Gecko's UA stylesheets
    // anyway.
    set_resources_path(Some("/files/mozilla/stylo/servo/resources/".to_owned()));

    // FIXME(bholley): Real window size.
    let window_size: TypedSize2D<ViewportPx, f32> = Size2D::typed(800.0, 600.0);
    let device = Device::new(MediaType::Screen, window_size);

    // FIXME(bholley): Real stylist and stylesheets.
    let stylesheets: Vec<Arc<Stylesheet>> = Vec::new();
    let mut stylist = Box::new(Stylist::new(device));
    let _needs_dirtying = stylist.update(&stylesheets, false);

    // FIXME(bholley): Hook this up to something.
    let new_animations_sender: Sender<Animation> = channel().0;

    let shared_style_context = SharedStyleContext {
        viewport_size: Size2D::new(Au(0), Au(0)),
        screen_size_changed: false,
        generation: 0,
        goal: ReflowGoal::ForScriptQuery,
        stylist: StylistWrapper(&*stylist),
        new_animations_sender: Mutex::new(new_animations_sender),
        running_animations: Arc::new(RwLock::new(HashMap::new())),
        expired_animations: Arc::new(RwLock::new(HashMap::new())),
        error_reporter: Box::new(StdoutErrorReporter),
    };

    let num_threads = cmp::max(num_cpus::get() * 3 / 4, 1);
    let mut parallel_traversal: WorkQueue<SharedStyleContext, WorkQueueData> =
        WorkQueue::new("StyleWorker", thread_state::LAYOUT, num_threads);

    if node.is_dirty() || node.has_dirty_descendants() {
        parallel::traverse_dom::<GeckoNode, RecalcStyleOnly>(node, &shared_style_context, &mut parallel_traversal);
    }

    parallel_traversal.shutdown();
}

#[no_mangle]
pub extern "C" fn Servo_DropNodeData(data: *mut ServoNodeData) -> () {
    unsafe {
        let _ = Box::<NonOpaqueStyleData>::from_raw(data as *mut NonOpaqueStyleData);
    }
}
