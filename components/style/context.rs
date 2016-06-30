/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The context within which style is calculated.

use animation::Animation;
use app_units::Au;
use dom::OpaqueNode;
use error_reporting::ParseErrorReporter;
use euclid::Size2D;
use matching::{ApplicableDeclarationsCache, StyleSharingCandidateCache};
use selector_impl::SelectorImplExt;
use selector_matching::Stylist;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

pub struct SharedStyleContext<Impl: SelectorImplExt> {
    /// The current viewport size.
    pub viewport_size: Size2D<Au>,

    /// Screen sized changed?
    pub screen_size_changed: bool,

    /// The CSS selector stylist.
    pub stylist: Arc<Stylist<Impl>>,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    pub generation: u32,

    /// Why is this reflow occurring
    pub goal: ReflowGoal,

    /// The animations that are currently running.
    pub running_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation<Impl>>>>>,

    /// The list of animations that have expired since the last style recalculation.
    pub expired_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation<Impl>>>>>,

    ///The CSS error reporter for all CSS loaded in this layout thread
    pub error_reporter: Box<ParseErrorReporter + Sync>,
}

pub struct LocalStyleContext<Impl: SelectorImplExt> {
    pub applicable_declarations_cache: RefCell<ApplicableDeclarationsCache<Impl::ComputedValues>>,
    pub style_sharing_candidate_cache: RefCell<StyleSharingCandidateCache<Impl::ComputedValues>>,
    /// A channel on which new animations that have been triggered by style
    /// recalculation can be sent.
    pub new_animations_sender: Sender<Animation<Impl>>,
}

pub trait StyleContext<'a, Impl: SelectorImplExt> {
    fn shared_context(&self) -> &'a SharedStyleContext<Impl>;
    fn local_context(&self) -> &LocalStyleContext<Impl>;
}

/// Why we're doing reflow.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ReflowGoal {
    /// We're reflowing in order to send a display list to the screen.
    ForDisplay,
    /// We're reflowing in order to satisfy a script query. No display list will be created.
    ForScriptQuery,
}
