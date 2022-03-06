use crate::dom::node::Node;
use app_units::Au;
use style::animation::DocumentAnimationSet;
use euclid::default::{Point2D, Rect};
use net_traits::image_cache::PendingImageId;
use style::invalidation::element::restyle_hints::RestyleHint;
use style::properties::PropertyId;
use msg::constellation_msg::BrowsingContextId;
use style::selector_parser::{PseudoElement, RestyleDamage, Snapshot};
use script_traits::WindowSizeData;
use servo_url::ImmutableOrigin;
use script_layout_interface::PendingImageState;
use crate::dom::node;

#[derive(Debug, PartialEq)]
pub enum NodesFromPointQueryType {
    All,
    Topmost,
}

#[derive(PartialEq)]
pub enum QueryMsg<'a> {
    ContentBoxQuery(&'a Node),
    ContentBoxesQuery(&'a Node),
    ClientRectQuery(&'a Node),
    NodeScrollGeometryQuery(&'a Node),
    OffsetParentQuery(&'a Node),
    TextIndexQuery(&'a Node, Point2D<f32>),
    NodesFromPointQuery(Point2D<f32>, NodesFromPointQueryType),
    NodeScrollIdQuery(&'a Node),
    ResolvedStyleQuery(&'a Node, Option<PseudoElement>, PropertyId),
    StyleQuery,
    ElementInnerTextQuery(&'a Node),
    ResolvedFontStyleQuery(&'a Node, PropertyId, String),
    InnerWindowDimensionsQuery(BrowsingContextId),
}

impl<'a> std::fmt::Debug for QueryMsg<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(match self {
            QueryMsg::ContentBoxQuery(..) => "ContentBoxQuery",
            QueryMsg::ContentBoxesQuery(..) => "ContentBoxesQuery",
            QueryMsg::ClientRectQuery(..) => "ClientRectQuery",
            QueryMsg::NodeScrollGeometryQuery(..) => "NodeScrollGeometryQuery",
            QueryMsg::OffsetParentQuery(..) => "OffsetParentQuery",
            QueryMsg::TextIndexQuery(..) => "TextIndexQuery",
            QueryMsg::NodesFromPointQuery(..) => "NodesFromPointQuery",
            QueryMsg::NodeScrollIdQuery(..) => "NodeScrollIdQuery",
            QueryMsg::ResolvedStyleQuery(..) => "ResolvedStyleQuery",
            QueryMsg::StyleQuery => "StyleQuery",
            QueryMsg::ElementInnerTextQuery(..) => "ElementInnerTextQuery",
            QueryMsg::ResolvedFontStyleQuery(..) => "ResolvedFontStyleQuery",
            QueryMsg::InnerWindowDimensionsQuery(..) => "InnerWindowDimensionsQuery",
        })
    }
}

/// Any query to perform with this reflow.
#[derive(Debug, PartialEq)]
pub enum ReflowGoal<'a> {
    Full,
    TickAnimations,
    LayoutQuery(QueryMsg<'a>, u64),
}

impl<'a> ReflowGoal<'a> {
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

/// The data associated with an image that is not yet present in the image cache.
/// Used by the script thread to hold on to DOM elements that need to be repainted
/// when an image fetch is complete.
pub struct PendingImage<'a> {
    pub state: PendingImageState,
    pub node: &'a Node,
    pub id: PendingImageId,
    pub origin: ImmutableOrigin,
}

impl<'a> From<script_layout_interface::PendingImage> for PendingImage<'a> {
    #[allow(unsafe_code)]
    fn from(image: script_layout_interface::PendingImage) -> PendingImage<'a> {
        // TODO: make opaque nodes use address of rust object so we don't need
        //       any JS conversion operations here.
        let node = unsafe { node::from_untrusted_node_address(image.node) };
        let node_ptr = &*node as *const Node;
        PendingImage {
            state: image.state,
            node: unsafe { &*node_ptr },
            id: image.id,
            origin: image.origin,
        }
    }
}

/// Information derived from a layout pass that needs to be returned to the script thread.
#[derive(Default)]
pub struct ReflowComplete<'a> {
    /// The list of images that were encountered that are in progress.
    pub pending_images: Vec<PendingImage<'a>>,
}

/// Information needed for a script-initiated reflow.
pub struct ScriptReflow<'a> {
    /// General reflow data.
    pub reflow_info: Reflow,
    /// The document node.
    pub document: &'a Node,
    /// The dirty root from which to restyle.
    pub dirty_root: Option<&'a Node>,
    /// Whether the document's stylesheets have changed since the last script reflow.
    pub stylesheets_changed: bool,
    /// The current window size.
    pub window_size: WindowSizeData,
    /// The goal of this reflow.
    pub reflow_goal: ReflowGoal<'a>,
    /// The number of objects in the dom #10110
    pub dom_count: u32,
    /// The current window origin
    pub origin: ImmutableOrigin,
    /// Restyle snapshot map.
    pub pending_restyles: Vec<(&'a Node, PendingRestyle)>,
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
