/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The context within which style is calculated.

use animation::Animation;
use app_units::Au;
use dom::OpaqueNode;
use error_reporting::ParseErrorReporter;
use euclid::Size2D;
#[cfg(feature = "servo")] use ipc_channel::ipc::IpcSender;
use matching::{ApplicableDeclarationsCache, StyleSharingCandidateCache};
#[cfg(feature = "servo")] use script_traits::ConstellationControlMsg;
use selector_matching::Stylist;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, RwLock};
use timer::Timer;

/// This structure is used to create a local style context from a shared one.
#[cfg(feature = "servo")]
pub struct LocalStyleContextCreationInfo {
    new_animations_sender: Sender<Animation>,
    script_chan: IpcSender<ConstellationControlMsg>,
}

#[cfg(feature = "servo")]
impl LocalStyleContextCreationInfo {
    pub fn new(animations_sender: Sender<Animation>, script_chan: IpcSender<ConstellationControlMsg>) -> Self {
        LocalStyleContextCreationInfo {
            new_animations_sender: animations_sender,
            script_chan: script_chan,
        }
    }
}
/// This structure is used to create a local style context from a shared one.
#[cfg(feature = "gecko")]
pub struct LocalStyleContextCreationInfo {
    new_animations_sender: Sender<Animation>,
}

#[cfg(feature = "gecko")]
impl LocalStyleContextCreationInfo {
    pub fn new(animations_sender: Sender<Animation>) -> Self {
        LocalStyleContextCreationInfo {
            new_animations_sender: animations_sender,
        }
    }
}

pub struct SharedStyleContext {
    /// The current viewport size.
    pub viewport_size: Size2D<Au>,

    /// Screen sized changed?
    pub screen_size_changed: bool,

    /// The CSS selector stylist.
    pub stylist: Arc<Stylist>,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    pub generation: u32,

    /// Why is this reflow occurring
    pub goal: ReflowGoal,

    /// The animations that are currently running.
    pub running_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,

    /// The list of animations that have expired since the last style recalculation.
    pub expired_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,

    ///The CSS error reporter for all CSS loaded in this layout thread
    pub error_reporter: Box<ParseErrorReporter + Sync>,

    /// Data needed to create the local style context from the shared one.
    pub local_context_creation_data: Mutex<LocalStyleContextCreationInfo>,

    /// The current timer for transitions and animations. This is needed to test
    /// them.
    pub timer: Timer,
}

#[cfg(feature = "servo")]
pub struct LocalStyleContext {
    pub applicable_declarations_cache: RefCell<ApplicableDeclarationsCache>,
    pub style_sharing_candidate_cache: RefCell<StyleSharingCandidateCache>,
    /// A channel on which new animations that have been triggered by style
    /// recalculation can be sent.
    pub new_animations_sender: Sender<Animation>,

    /// A channel to communicate with the script thread about transition events.
    pub script_chan: IpcSender<ConstellationControlMsg>,
}

#[cfg(feature = "gecko")]
pub struct LocalStyleContext {
    pub applicable_declarations_cache: RefCell<ApplicableDeclarationsCache>,
    pub style_sharing_candidate_cache: RefCell<StyleSharingCandidateCache>,
    /// A channel on which new animations that have been triggered by style
    /// recalculation can be sent.
    pub new_animations_sender: Sender<Animation>,
}

#[cfg(feature = "servo")]
impl LocalStyleContext {
    pub fn new(local_context_creation_data: &LocalStyleContextCreationInfo) -> Self {
        LocalStyleContext {
            applicable_declarations_cache: RefCell::new(ApplicableDeclarationsCache::new()),
            style_sharing_candidate_cache: RefCell::new(StyleSharingCandidateCache::new()),
            new_animations_sender: local_context_creation_data.new_animations_sender.clone(),
            script_chan: local_context_creation_data.script_chan.clone(),
        }
    }
}

#[cfg(feature = "gecko")]
impl LocalStyleContext {
    pub fn new(local_context_creation_data: &LocalStyleContextCreationInfo) -> Self {
        LocalStyleContext {
            applicable_declarations_cache: RefCell::new(ApplicableDeclarationsCache::new()),
            style_sharing_candidate_cache: RefCell::new(StyleSharingCandidateCache::new()),
            new_animations_sender: local_context_creation_data.new_animations_sender.clone(),
        }
    }
}

pub trait StyleContext<'a> {
    fn shared_context(&self) -> &'a SharedStyleContext;
    fn local_context(&self) -> &LocalStyleContext;
}

/// Why we're doing reflow.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ReflowGoal {
    /// We're reflowing in order to send a display list to the screen.
    ForDisplay,
    /// We're reflowing in order to satisfy a script query. No display list will be created.
    ForScriptQuery,
}
