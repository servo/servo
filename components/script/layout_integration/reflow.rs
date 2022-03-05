use app_units::Au;
use style::dom::OpaqueNode;
use script_layout_interface::{PendingImage, TrustedNodeAddress};
use style::animation::DocumentAnimationSet;
use euclid::default::{Point2D, Rect};
use style::invalidation::element::restyle_hints::RestyleHint;
use style::properties::PropertyId;
use msg::constellation_msg::BrowsingContextId;
use style::selector_parser::{PseudoElement, RestyleDamage, Snapshot};
use script_traits::WindowSizeData;
use crossbeam_channel::Sender;
use servo_url::{ImmutableOrigin, ServoUrl};

#[derive(Debug, PartialEq)]
pub enum NodesFromPointQueryType {
    All,
    Topmost,
}

#[derive(Debug, PartialEq)]
pub enum QueryMsg {
    ContentBoxQuery(OpaqueNode),
    ContentBoxesQuery(OpaqueNode),
    ClientRectQuery(OpaqueNode),
    NodeScrollGeometryQuery(OpaqueNode),
    OffsetParentQuery(OpaqueNode),
    TextIndexQuery(OpaqueNode, Point2D<f32>),
    NodesFromPointQuery(Point2D<f32>, NodesFromPointQueryType),

    // FIXME(nox): The following queries use the TrustedNodeAddress to
    // access actual DOM nodes, but those values can be constructed from
    // garbage values such as `0xdeadbeef as *const _`, this is unsound.
    NodeScrollIdQuery(TrustedNodeAddress),
    ResolvedStyleQuery(TrustedNodeAddress, Option<PseudoElement>, PropertyId),
    StyleQuery,
    ElementInnerTextQuery(TrustedNodeAddress),
    ResolvedFontStyleQuery(TrustedNodeAddress, PropertyId, String),
    InnerWindowDimensionsQuery(BrowsingContextId),
}

/// Any query to perform with this reflow.
#[derive(Debug, PartialEq)]
pub enum ReflowGoal {
    Full,
    TickAnimations,
    LayoutQuery(QueryMsg, u64),
}

impl ReflowGoal {
    /// Returns true if the given ReflowQuery needs a full, up-to-date display list to
    /// be present or false if it only needs stacking-relative positions.
    pub fn needs_display_list(&self) -> bool {
        match *self {
            ReflowGoal::Full | ReflowGoal::TickAnimations => true,
            ReflowGoal::LayoutQuery(ref querymsg, _) => match *querymsg {
                QueryMsg::NodesFromPointQuery(..) |
                QueryMsg::TextIndexQuery(..) |
                QueryMsg::InnerWindowDimensionsQuery(_) |
                QueryMsg::ElementInnerTextQuery(_) => true,
                QueryMsg::ContentBoxQuery(_) |
                QueryMsg::ContentBoxesQuery(_) |
                QueryMsg::ClientRectQuery(_) |
                QueryMsg::NodeScrollGeometryQuery(_) |
                QueryMsg::NodeScrollIdQuery(_) |
                QueryMsg::ResolvedStyleQuery(..) |
                QueryMsg::ResolvedFontStyleQuery(..) |
                QueryMsg::OffsetParentQuery(_) |
                QueryMsg::StyleQuery => false,
            },
        }
    }

    /// Returns true if the given ReflowQuery needs its display list send to WebRender or
    /// false if a layout_thread display list is sufficient.
    pub fn needs_display(&self) -> bool {
        match *self {
            ReflowGoal::Full | ReflowGoal::TickAnimations => true,
            ReflowGoal::LayoutQuery(ref querymsg, _) => match *querymsg {
                QueryMsg::NodesFromPointQuery(..) |
                QueryMsg::TextIndexQuery(..) |
                QueryMsg::ElementInnerTextQuery(_) => true,
                QueryMsg::ContentBoxQuery(_) |
                QueryMsg::ContentBoxesQuery(_) |
                QueryMsg::ClientRectQuery(_) |
                QueryMsg::NodeScrollGeometryQuery(_) |
                QueryMsg::NodeScrollIdQuery(_) |
                QueryMsg::ResolvedStyleQuery(..) |
                QueryMsg::ResolvedFontStyleQuery(..) |
                QueryMsg::OffsetParentQuery(_) |
                QueryMsg::InnerWindowDimensionsQuery(_) |
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

impl PendingRestyle {
    /// Creates a new empty pending restyle.
    #[inline]
    pub fn new() -> Self {
        PendingRestyle {
            snapshot: None,
            hint: RestyleHint::empty(),
            damage: RestyleDamage::empty(),
        }
    }
}
