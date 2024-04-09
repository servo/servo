/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use crossbeam_channel::Sender;
use euclid::default::Rect;
use malloc_size_of_derive::MallocSizeOf;
use profile_traits::mem::ReportsChan;
use script_traits::{Painter, ScrollState, WindowSizeData};
use servo_atoms::Atom;
use servo_url::ImmutableOrigin;
use style::animation::DocumentAnimationSet;
use style::context::QuirksMode;
use style::invalidation::element::restyle_hints::RestyleHint;
use style::selector_parser::{RestyleDamage, Snapshot};

use crate::{PendingImage, TrustedNodeAddress};

/// Asynchronous messages that script can send to layout.
pub enum Msg {
    /// Change the quirks mode.
    SetQuirksMode(QuirksMode),

    /// Requests a reflow.
    Reflow(ScriptReflow),

    /// Requests that layout measure its memory usage. The resulting reports are sent back
    /// via the supplied channel.
    CollectReports(ReportsChan),

    /// Requests that layout immediately shut down. There must be no more nodes left after
    /// this, or layout will crash.
    ExitNow,

    /// Tells layout about the new scrolling offsets of each scrollable stacking context.
    SetScrollStates(Vec<ScrollState>),

    /// Tells layout that script has added some paint worklet modules.
    RegisterPaint(Atom, Vec<Atom>, Box<dyn Painter>),
}

#[derive(Debug, PartialEq)]
pub enum NodesFromPointQueryType {
    All,
    Topmost,
}

#[derive(Debug, PartialEq)]
pub enum QueryMsg {
    ContentBox,
    ContentBoxes,
    ClientRectQuery,
    ScrollingAreaQuery,
    OffsetParentQuery,
    TextIndexQuery,
    NodesFromPointQuery,
    ResolvedStyleQuery,
    StyleQuery,
    ElementInnerTextQuery,
    ResolvedFontStyleQuery,
    InnerWindowDimensionsQuery,
}

/// Any query to perform with this reflow.
#[derive(Debug, PartialEq)]
pub enum ReflowGoal {
    Full,
    TickAnimations,
    LayoutQuery(QueryMsg, u64),

    /// Tells layout about a single new scrolling offset from the script. The rest will
    /// remain untouched and layout won't forward this back to script.
    UpdateScrollNode(ScrollState),
}

impl ReflowGoal {
    /// Returns true if the given ReflowQuery needs a full, up-to-date display list to
    /// be present or false if it only needs stacking-relative positions.
    pub fn needs_display_list(&self) -> bool {
        match *self {
            ReflowGoal::Full | ReflowGoal::TickAnimations | ReflowGoal::UpdateScrollNode(_) => true,
            ReflowGoal::LayoutQuery(ref querymsg, _) => match *querymsg {
                QueryMsg::ElementInnerTextQuery |
                QueryMsg::InnerWindowDimensionsQuery |
                QueryMsg::NodesFromPointQuery |
                QueryMsg::ResolvedStyleQuery |
                QueryMsg::TextIndexQuery => true,
                QueryMsg::ClientRectQuery |
                QueryMsg::ContentBox |
                QueryMsg::ContentBoxes |
                QueryMsg::OffsetParentQuery |
                QueryMsg::ResolvedFontStyleQuery |
                QueryMsg::ScrollingAreaQuery |
                QueryMsg::StyleQuery => false,
            },
        }
    }

    /// Returns true if the given ReflowQuery needs its display list send to WebRender or
    /// false if a layout_thread display list is sufficient.
    pub fn needs_display(&self) -> bool {
        match *self {
            ReflowGoal::Full | ReflowGoal::TickAnimations | ReflowGoal::UpdateScrollNode(_) => true,
            ReflowGoal::LayoutQuery(ref querymsg, _) => match *querymsg {
                QueryMsg::NodesFromPointQuery |
                QueryMsg::TextIndexQuery |
                QueryMsg::ElementInnerTextQuery => true,
                QueryMsg::ContentBox |
                QueryMsg::ContentBoxes |
                QueryMsg::ClientRectQuery |
                QueryMsg::ScrollingAreaQuery |
                QueryMsg::ResolvedStyleQuery |
                QueryMsg::ResolvedFontStyleQuery |
                QueryMsg::OffsetParentQuery |
                QueryMsg::InnerWindowDimensionsQuery |
                QueryMsg::StyleQuery => false,
            },
        }
    }
}

/// Information needed for a reflow.
pub struct Reflow {
    ///  A clipping rectangle for the page, an enlarged rectangle containing the viewport.
    pub page_clip_rect: Rect<Au>,
}

/// Information derived from a layout pass that needs to be returned to the script thread.
#[derive(Default)]
pub struct ReflowComplete {
    /// The list of images that were encountered that are in progress.
    pub pending_images: Vec<PendingImage>,
}

/// Information needed for a script-initiated reflow.
pub struct ScriptReflow {
    /// General reflow data.
    pub reflow_info: Reflow,
    /// The document node.
    pub document: TrustedNodeAddress,
    /// The dirty root from which to restyle.
    pub dirty_root: Option<TrustedNodeAddress>,
    /// Whether the document's stylesheets have changed since the last script reflow.
    pub stylesheets_changed: bool,
    /// The current window size.
    pub window_size: WindowSizeData,
    /// The channel that we send a notification to.
    pub script_join_chan: Sender<ReflowComplete>,
    /// The goal of this reflow.
    pub reflow_goal: ReflowGoal,
    /// The number of objects in the dom #10110
    pub dom_count: u32,
    /// The current window origin
    pub origin: ImmutableOrigin,
    /// Restyle snapshot map.
    pub pending_restyles: Vec<(TrustedNodeAddress, PendingRestyle)>,
    /// The current animation timeline value.
    pub animation_timeline_value: f64,
    /// The set of animations for this document.
    pub animations: DocumentAnimationSet,
}

/// A pending restyle.
#[derive(Debug, MallocSizeOf)]
pub struct PendingRestyle {
    /// If this element had a state or attribute change since the last restyle, track
    /// the original condition of the element.
    pub snapshot: Option<Snapshot>,

    /// Any explicit restyles hints that have been accumulated for this element.
    pub hint: RestyleHint,

    /// Any explicit restyles damage that have been accumulated for this element.
    pub damage: RestyleDamage,
}

impl Default for PendingRestyle {
    /// Creates a new empty pending restyle.
    #[inline]
    fn default() -> Self {
        Self {
            snapshot: None,
            hint: RestyleHint::empty(),
            damage: RestyleDamage::empty(),
        }
    }
}
