/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Fragment` type, which represents the leaves of the layout tree.

#![deny(unsafe_code)]

use StyleArc;
use app_units::Au;
use canvas_traits::CanvasMsg;
use context::{LayoutContext, with_thread_local_font_context};
use euclid::{Matrix4D, Point2D, Radians, Rect, Size2D};
use floats::ClearType;
use flow::{self, ImmutableFlowUtils};
use flow_ref::FlowRef;
use gfx;
use gfx::display_list::{BLUR_INFLATION_FACTOR, OpaqueNode};
use gfx::text::glyph::ByteIndex;
use gfx::text::text_run::{TextRun, TextRunSlice};
use gfx_traits::StackingContextId;
use inline::{FIRST_FRAGMENT_OF_ELEMENT, InlineFragmentContext, InlineFragmentNodeInfo};
use inline::{InlineMetrics, LAST_FRAGMENT_OF_ELEMENT, LineMetrics};
use ipc_channel::ipc::IpcSender;
#[cfg(debug_assertions)]
use layout_debug;
use model::{self, IntrinsicISizes, IntrinsicISizesContribution, MaybeAuto, SizeConstraint};
use model::{style_length, ToGfxMatrix};
use msg::constellation_msg::PipelineId;
use net_traits::image::base::{Image, ImageMetadata};
use net_traits::image_cache::{ImageOrMetadataAvailable, UsePlaceholder};
use range::*;
use script_layout_interface::HTMLCanvasData;
use script_layout_interface::SVGSVGData;
use script_layout_interface::wrapper_traits::{PseudoElementType, ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use servo_url::ServoUrl;
use std::{f32, fmt};
use std::borrow::ToOwned;
use std::cmp::{Ordering, max, min};
use std::collections::LinkedList;
use std::sync::{Arc, Mutex};
use style::computed_values::{border_collapse, box_sizing, clear, color, display, mix_blend_mode};
use style::computed_values::{overflow_wrap, overflow_x, position, text_decoration_line, transform};
use style::computed_values::{transform_style, vertical_align, white_space, word_break};
use style::computed_values::content::ContentItem;
use style::logical_geometry::{Direction, LogicalMargin, LogicalRect, LogicalSize, WritingMode};
use style::properties::ServoComputedValues;
use style::selector_parser::RestyleDamage;
use style::servo::restyle_damage::RECONSTRUCT_FLOW;
use style::str::char_is_whitespace;
use style::values::{self, Either, Auto};
use style::values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto};
use text;
use text::TextRunScanner;
use wrapper::ThreadSafeLayoutNodeHelpers;

// From gfxFontConstants.h in Firefox.
static FONT_SUBSCRIPT_OFFSET_RATIO: f32 = 0.20;
static FONT_SUPERSCRIPT_OFFSET_RATIO: f32 = 0.34;

// https://drafts.csswg.org/css-images/#default-object-size
static DEFAULT_REPLACED_WIDTH: i32 = 300;
static DEFAULT_REPLACED_HEIGHT: i32 = 150;

/// Fragments (`struct Fragment`) are the leaves of the layout tree. They cannot position
/// themselves. In general, fragments do not have a simple correspondence with CSS fragments in the
/// specification:
///
/// * Several fragments may correspond to the same CSS box or DOM node. For example, a CSS text box
/// broken across two lines is represented by two fragments.
///
/// * Some CSS fragments are not created at all, such as some anonymous block fragments induced by
///   inline fragments with block-level sibling fragments. In that case, Servo uses an `InlineFlow`
///   with `BlockFlow` siblings; the `InlineFlow` is block-level, but not a block container. It is
///   positioned as if it were a block fragment, but its children are positioned according to
///   inline flow.
///
/// A `SpecificFragmentInfo::Generic` is an empty fragment that contributes only borders, margins,
/// padding, and backgrounds. It is analogous to a CSS nonreplaced content box.
///
/// A fragment's type influences how its styles are interpreted during layout. For example,
/// replaced content such as images are resized differently from tables, text, or other content.
/// Different types of fragments may also contain custom data; for example, text fragments contain
/// text.
///
/// Do not add fields to this structure unless they're really really mega necessary! Fragments get
/// moved around a lot and thus their size impacts performance of layout quite a bit.
///
/// FIXME(#2260, pcwalton): This can be slimmed down some by (at least) moving `inline_context`
/// to be on `InlineFlow` only.
#[derive(Clone)]
pub struct Fragment {
    /// An opaque reference to the DOM node that this `Fragment` originates from.
    pub node: OpaqueNode,

    /// The CSS style of this fragment.
    pub style: StyleArc<ServoComputedValues>,

    /// The CSS style of this fragment when it's selected
    pub selected_style: StyleArc<ServoComputedValues>,

    /// The position of this fragment relative to its owning flow. The size includes padding and
    /// border, but not margin.
    ///
    /// NB: This does not account for relative positioning.
    /// NB: Collapsed borders are not included in this.
    pub border_box: LogicalRect<Au>,

    /// The sum of border and padding; i.e. the distance from the edge of the border box to the
    /// content edge of the fragment.
    pub border_padding: LogicalMargin<Au>,

    /// The margin of the content box.
    pub margin: LogicalMargin<Au>,

    /// Info specific to the kind of fragment. Keep this enum small.
    pub specific: SpecificFragmentInfo,

    /// Holds the style context information for fragments that are part of an inline formatting
    /// context.
    pub inline_context: Option<InlineFragmentContext>,

    /// How damaged this fragment is since last reflow.
    pub restyle_damage: RestyleDamage,

    /// The pseudo-element that this fragment represents.
    pub pseudo: PseudoElementType<()>,

    /// Various flags for this fragment.
    pub flags: FragmentFlags,

    /// A debug ID that is consistent for the life of this fragment (via transform etc).
    /// This ID should not be considered stable across multiple layouts or fragment
    /// manipulations.
    debug_id: DebugId,

    /// The ID of the StackingContext that contains this fragment. This is initialized
    /// to 0, but it assigned during the collect_stacking_contexts phase of display
    /// list construction.
    pub stacking_context_id: StackingContextId,
}

impl Serialize for Fragment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = try!(serializer.serialize_struct("fragment", 3));
        try!(serializer.serialize_field("id", &self.debug_id));
        try!(serializer.serialize_field("border_box", &self.border_box));
        try!(serializer.serialize_field("margin", &self.margin));
        serializer.end()
    }
}

/// Info specific to the kind of fragment.
///
/// Keep this enum small. As in, no more than one word. Or pcwalton will yell at you.
#[derive(Clone)]
pub enum SpecificFragmentInfo {
    Generic,

    /// A piece of generated content that cannot be resolved into `ScannedText` until the generated
    /// content resolution phase (e.g. an ordered list item marker).
    GeneratedContent(Box<GeneratedContentInfo>),

    Iframe(IframeFragmentInfo),
    Image(Box<ImageFragmentInfo>),
    Canvas(Box<CanvasFragmentInfo>),
    Svg(Box<SvgFragmentInfo>),

    /// A hypothetical box (see CSS 2.1 § 10.3.7) for an absolutely-positioned block that was
    /// declared with `display: inline;`.
    InlineAbsoluteHypothetical(InlineAbsoluteHypotheticalFragmentInfo),

    InlineBlock(InlineBlockFragmentInfo),

    /// An inline fragment that establishes an absolute containing block for its descendants (i.e.
    /// a positioned inline fragment).
    InlineAbsolute(InlineAbsoluteFragmentInfo),

    ScannedText(Box<ScannedTextFragmentInfo>),
    Table,
    TableCell,
    TableColumn(TableColumnFragmentInfo),
    TableRow,
    TableWrapper,
    Multicol,
    MulticolColumn,
    UnscannedText(Box<UnscannedTextFragmentInfo>),

    /// A container for a fragment that got truncated by text-overflow.
    /// "Totally truncated fragments" are not rendered at all.
    /// Text fragments may be partially truncated (in which case this renders like a text fragment).
    /// Other fragments can only be totally truncated or not truncated at all.
    TruncatedFragment(Box<TruncatedFragmentInfo>),
}

impl SpecificFragmentInfo {
    fn restyle_damage(&self) -> RestyleDamage {
        let flow =
            match *self {
                SpecificFragmentInfo::Canvas(_) |
                SpecificFragmentInfo::GeneratedContent(_) |
                SpecificFragmentInfo::Iframe(_) |
                SpecificFragmentInfo::Image(_) |
                SpecificFragmentInfo::ScannedText(_) |
                SpecificFragmentInfo::Svg(_) |
                SpecificFragmentInfo::Table |
                SpecificFragmentInfo::TableCell |
                SpecificFragmentInfo::TableColumn(_) |
                SpecificFragmentInfo::TableRow |
                SpecificFragmentInfo::TableWrapper |
                SpecificFragmentInfo::Multicol |
                SpecificFragmentInfo::MulticolColumn |
                SpecificFragmentInfo::UnscannedText(_) |
                SpecificFragmentInfo::TruncatedFragment(_) |
                SpecificFragmentInfo::Generic => return RestyleDamage::empty(),
                SpecificFragmentInfo::InlineAbsoluteHypothetical(ref info) => &info.flow_ref,
                SpecificFragmentInfo::InlineAbsolute(ref info) => &info.flow_ref,
                SpecificFragmentInfo::InlineBlock(ref info) => &info.flow_ref,
            };

        flow::base(&**flow).restyle_damage
    }

    pub fn get_type(&self) -> &'static str {
        match *self {
            SpecificFragmentInfo::Canvas(_) => "SpecificFragmentInfo::Canvas",
            SpecificFragmentInfo::Generic => "SpecificFragmentInfo::Generic",
            SpecificFragmentInfo::GeneratedContent(_) => "SpecificFragmentInfo::GeneratedContent",
            SpecificFragmentInfo::Iframe(_) => "SpecificFragmentInfo::Iframe",
            SpecificFragmentInfo::Image(_) => "SpecificFragmentInfo::Image",
            SpecificFragmentInfo::InlineAbsolute(_) => "SpecificFragmentInfo::InlineAbsolute",
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) => {
                "SpecificFragmentInfo::InlineAbsoluteHypothetical"
            }
            SpecificFragmentInfo::InlineBlock(_) => "SpecificFragmentInfo::InlineBlock",
            SpecificFragmentInfo::ScannedText(_) => "SpecificFragmentInfo::ScannedText",
            SpecificFragmentInfo::Svg(_) => "SpecificFragmentInfo::Svg",
            SpecificFragmentInfo::Table => "SpecificFragmentInfo::Table",
            SpecificFragmentInfo::TableCell => "SpecificFragmentInfo::TableCell",
            SpecificFragmentInfo::TableColumn(_) => "SpecificFragmentInfo::TableColumn",
            SpecificFragmentInfo::TableRow => "SpecificFragmentInfo::TableRow",
            SpecificFragmentInfo::TableWrapper => "SpecificFragmentInfo::TableWrapper",
            SpecificFragmentInfo::Multicol => "SpecificFragmentInfo::Multicol",
            SpecificFragmentInfo::MulticolColumn => "SpecificFragmentInfo::MulticolColumn",
            SpecificFragmentInfo::UnscannedText(_) => "SpecificFragmentInfo::UnscannedText",
            SpecificFragmentInfo::TruncatedFragment(_) => "SpecificFragmentInfo::TruncatedFragment"
        }
    }
}

impl fmt::Debug for SpecificFragmentInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpecificFragmentInfo::ScannedText(ref info) => write!(f, "{:?}", info.text()),
            SpecificFragmentInfo::UnscannedText(ref info) => write!(f, "{:?}", info.text),
            _ => Ok(())
        }
    }
}

/// Information for generated content.
#[derive(Clone)]
pub enum GeneratedContentInfo {
    ListItem,
    ContentItem(ContentItem),
    /// Placeholder for elements with generated content that did not generate any fragments.
    Empty,
}

/// A hypothetical box (see CSS 2.1 § 10.3.7) for an absolutely-positioned block that was declared
/// with `display: inline;`.
///
/// FIXME(pcwalton): Stop leaking this `FlowRef` to layout; that is not memory safe because layout
/// can clone it.
#[derive(Clone)]
pub struct InlineAbsoluteHypotheticalFragmentInfo {
    pub flow_ref: FlowRef,
}

impl InlineAbsoluteHypotheticalFragmentInfo {
    pub fn new(flow_ref: FlowRef) -> InlineAbsoluteHypotheticalFragmentInfo {
        InlineAbsoluteHypotheticalFragmentInfo {
            flow_ref: flow_ref,
        }
    }
}

/// A fragment that represents an inline-block element.
///
/// FIXME(pcwalton): Stop leaking this `FlowRef` to layout; that is not memory safe because layout
/// can clone it.
#[derive(Clone)]
pub struct InlineBlockFragmentInfo {
    pub flow_ref: FlowRef,
}

impl InlineBlockFragmentInfo {
    pub fn new(flow_ref: FlowRef) -> InlineBlockFragmentInfo {
        InlineBlockFragmentInfo {
            flow_ref: flow_ref,
        }
    }
}

/// An inline fragment that establishes an absolute containing block for its descendants (i.e.
/// a positioned inline fragment).
///
/// FIXME(pcwalton): Stop leaking this `FlowRef` to layout; that is not memory safe because layout
/// can clone it.
#[derive(Clone)]
pub struct InlineAbsoluteFragmentInfo {
    pub flow_ref: FlowRef,
}

impl InlineAbsoluteFragmentInfo {
    pub fn new(flow_ref: FlowRef) -> InlineAbsoluteFragmentInfo {
        InlineAbsoluteFragmentInfo {
            flow_ref: flow_ref,
        }
    }
}

#[derive(Clone)]
pub struct CanvasFragmentInfo {
    pub ipc_renderer: Option<Arc<Mutex<IpcSender<CanvasMsg>>>>,
    pub dom_width: Au,
    pub dom_height: Au,
}

impl CanvasFragmentInfo {
    pub fn new(data: HTMLCanvasData) -> CanvasFragmentInfo {
        CanvasFragmentInfo {
            ipc_renderer: data.ipc_renderer
                              .map(|renderer| Arc::new(Mutex::new(renderer))),
            dom_width: Au::from_px(data.width as i32),
            dom_height: Au::from_px(data.height as i32),
        }
    }
}

#[derive(Clone)]
pub struct SvgFragmentInfo {
    pub dom_width: Au,
    pub dom_height: Au,
}

impl SvgFragmentInfo {
    pub fn new(data: SVGSVGData) -> SvgFragmentInfo {
        SvgFragmentInfo {
            dom_width: Au::from_px(data.width as i32),
            dom_height: Au::from_px(data.height as i32),
        }
    }
}


/// A fragment that represents a replaced content image and its accompanying borders, shadows, etc.
#[derive(Clone)]
pub struct ImageFragmentInfo {
    pub image: Option<Arc<Image>>,
    pub metadata: Option<ImageMetadata>,
}

impl ImageFragmentInfo {
    /// Creates a new image fragment from the given URL and local image cache.
    ///
    /// FIXME(pcwalton): The fact that image fragments store the cache in the fragment makes little
    /// sense to me.
    pub fn new<N: ThreadSafeLayoutNode>(url: Option<ServoUrl>,
                                        node: &N,
                                        layout_context: &LayoutContext)
               -> ImageFragmentInfo {
        let image_or_metadata = url.and_then(|url| {
            layout_context.get_or_request_image_or_meta(node.opaque(),
                                                        url,
                                                        UsePlaceholder::Yes)
        });

        let (image, metadata) = match image_or_metadata {
            Some(ImageOrMetadataAvailable::ImageAvailable(i)) => {
                (Some(i.clone()), Some(ImageMetadata { height: i.height, width: i.width } ))
            }
            Some(ImageOrMetadataAvailable::MetadataAvailable(m)) => {
                (None, Some(m))
            }
            None => {
                (None, None)
            }
        };

        ImageFragmentInfo {
            image: image,
            metadata: metadata,
        }
    }

    pub fn tile_image_round(position: &mut Au,
                            size: &mut Au,
                            absolute_anchor_origin: Au,
                            image_size: &mut Au) {
        if *size == Au(0) || *image_size == Au(0) {
            *position = Au(0);
            *size =Au(0);
            return;
        }

        let number_of_tiles = (size.to_f32_px() / image_size.to_f32_px()).round().max(1.0);
        *image_size = *size / (number_of_tiles as i32);
        ImageFragmentInfo::tile_image(position, size, absolute_anchor_origin, *image_size);
    }

    pub fn tile_image_spaced(position: &mut Au,
                             size: &mut Au,
                             tile_spacing: &mut Au,
                             absolute_anchor_origin: Au,
                             image_size: Au) {
        if *size == Au(0) || image_size == Au(0) {
            *position = Au(0);
            *size = Au(0);
            *tile_spacing = Au(0);
            return;
        }

        // Per the spec, if the space available is not enough for two images, just tile as
        // normal but only display a single tile.
        if image_size * 2 >= *size {
            ImageFragmentInfo::tile_image(position,
                                          size,
                                          absolute_anchor_origin,
                                          image_size);
            *tile_spacing = Au(0);
            *size = image_size;
            return;
        }

        // Take the box size, remove room for two tiles on the edges, and then calculate how many
        // other tiles fit in between them.
        let size_remaining = *size - (image_size * 2);
        let num_middle_tiles = (size_remaining.to_f32_px() / image_size.to_f32_px()).floor() as i32;

        // Allocate the remaining space as padding between tiles. background-position is ignored
        // as per the spec, so the position is just the box origin. We are also ignoring
        // background-attachment here, which seems unspecced when combined with
        // background-repeat: space.
        let space_for_middle_tiles = image_size * num_middle_tiles;
        *tile_spacing = (size_remaining - space_for_middle_tiles) / (num_middle_tiles + 1);
    }

    /// Tile an image
    pub fn tile_image(position: &mut Au,
                      size: &mut Au,
                      absolute_anchor_origin: Au,
                      image_size: Au) {
        // Avoid division by zero below!
        if image_size == Au(0) {
            return
        }

        let delta_pixels = absolute_anchor_origin - *position;
        let image_size_px = image_size.to_f32_px();
        let tile_count = ((delta_pixels.to_f32_px() + image_size_px - 1.0) / image_size_px).floor();
        let offset = image_size * (tile_count as i32);
        let new_position = absolute_anchor_origin - offset;
        *size = *position - new_position + *size;
        *position = new_position;
    }
}

/// A fragment that represents an inline frame (iframe). This stores the pipeline ID so that the
/// size of this iframe can be communicated via the constellation to the iframe's own layout thread.
#[derive(Clone)]
pub struct IframeFragmentInfo {
    /// The pipeline ID of this iframe.
    pub pipeline_id: PipelineId,
}

impl IframeFragmentInfo {
    /// Creates the information specific to an iframe fragment.
    pub fn new<N: ThreadSafeLayoutNode>(node: &N) -> IframeFragmentInfo {
        let pipeline_id = node.iframe_pipeline_id();
        IframeFragmentInfo {
            pipeline_id: pipeline_id,
        }
    }
}

/// A scanned text fragment represents a single run of text with a distinct style. A `TextFragment`
/// may be split into two or more fragments across line breaks. Several `TextFragment`s may
/// correspond to a single DOM text node. Split text fragments are implemented by referring to
/// subsets of a single `TextRun` object.
#[derive(Clone)]
pub struct ScannedTextFragmentInfo {
    /// The text run that this represents.
    pub run: Arc<TextRun>,

    /// The intrinsic size of the text fragment.
    pub content_size: LogicalSize<Au>,

    /// The byte offset of the insertion point, if any.
    pub insertion_point: Option<ByteIndex>,

    /// The range within the above text run that this represents.
    pub range: Range<ByteIndex>,

    /// The endpoint of the above range, including whitespace that was stripped out. This exists
    /// so that we can restore the range to its original value (before line breaking occurred) when
    /// performing incremental reflow.
    pub range_end_including_stripped_whitespace: ByteIndex,

    pub flags: ScannedTextFlags,
}

bitflags! {
    pub flags ScannedTextFlags: u8 {
        /// Whether a line break is required after this fragment if wrapping on newlines (e.g. if
        /// `white-space: pre` is in effect).
        const REQUIRES_LINE_BREAK_AFTERWARD_IF_WRAPPING_ON_NEWLINES = 0x01,

        /// Is this fragment selected?
        const SELECTED = 0x02,
    }
}

impl ScannedTextFragmentInfo {
    /// Creates the information specific to a scanned text fragment from a range and a text run.
    pub fn new(run: Arc<TextRun>,
               range: Range<ByteIndex>,
               content_size: LogicalSize<Au>,
               insertion_point: Option<ByteIndex>,
               flags: ScannedTextFlags)
               -> ScannedTextFragmentInfo {
        ScannedTextFragmentInfo {
            run: run,
            range: range,
            insertion_point: insertion_point,
            content_size: content_size,
            range_end_including_stripped_whitespace: range.end(),
            flags: flags,
        }
    }

    pub fn text(&self) -> &str {
        &self.run.text[self.range.begin().to_usize() .. self.range.end().to_usize()]
    }

    pub fn requires_line_break_afterward_if_wrapping_on_newlines(&self) -> bool {
        self.flags.contains(REQUIRES_LINE_BREAK_AFTERWARD_IF_WRAPPING_ON_NEWLINES)
    }

    pub fn selected(&self) -> bool {
        self.flags.contains(SELECTED)
    }
}

/// Describes how to split a fragment. This is used during line breaking as part of the return
/// value of `find_split_info_for_inline_size()`.
#[derive(Debug, Clone)]
pub struct SplitInfo {
    // TODO(bjz): this should only need to be a single character index, but both values are
    // currently needed for splitting in the `inline::try_append_*` functions.
    pub range: Range<ByteIndex>,
    pub inline_size: Au,
}

impl SplitInfo {
    fn new(range: Range<ByteIndex>, info: &ScannedTextFragmentInfo) -> SplitInfo {
        let inline_size = info.run.advance_for_range(&range);
        SplitInfo {
            range: range,
            inline_size: inline_size,
        }
    }
}

/// Describes how to split a fragment into two. This contains up to two `SplitInfo`s.
pub struct SplitResult {
    /// The part of the fragment that goes on the first line.
    pub inline_start: Option<SplitInfo>,
    /// The part of the fragment that goes on the second line.
    pub inline_end: Option<SplitInfo>,
    /// The text run which is being split.
    pub text_run: Arc<TextRun>,
}

/// Describes how a fragment should be truncated.
struct TruncationResult {
    /// The part of the fragment remaining after truncation.
    split: SplitInfo,
    /// The text run which is being truncated.
    text_run: Arc<TextRun>,
}

/// Data for an unscanned text fragment. Unscanned text fragments are the results of flow
/// construction that have not yet had their inline-size determined.
#[derive(Clone)]
pub struct UnscannedTextFragmentInfo {
    /// The text inside the fragment.
    pub text: Box<str>,

    /// The selected text range.  An empty range represents the insertion point.
    pub selection: Option<Range<ByteIndex>>,
}

impl UnscannedTextFragmentInfo {
    /// Creates a new instance of `UnscannedTextFragmentInfo` from the given text.
    #[inline]
    pub fn new(text: String, selection: Option<Range<ByteIndex>>) -> UnscannedTextFragmentInfo {
        UnscannedTextFragmentInfo {
            text: text.into_boxed_str(),
            selection: selection,
        }
    }
}

/// A fragment that represents a table column.
#[derive(Copy, Clone)]
pub struct TableColumnFragmentInfo {
    /// the number of columns a <col> element should span
    pub span: u32,
}

impl TableColumnFragmentInfo {
    /// Create the information specific to an table column fragment.
    pub fn new<N: ThreadSafeLayoutNode>(node: &N) -> TableColumnFragmentInfo {
        let element = node.as_element().unwrap();
        let span = element.get_attr(&ns!(), &local_name!("span"))
                          .and_then(|string| string.parse().ok())
                          .unwrap_or(0);
        TableColumnFragmentInfo {
            span: span,
        }
    }
}

/// A wrapper for fragments that have been truncated by the `text-overflow` property.
/// This may have an associated text node, or, if the fragment was completely truncated,
/// it may act as an invisible marker for incremental reflow.
#[derive(Clone)]
pub struct TruncatedFragmentInfo {
    pub text_info: Option<ScannedTextFragmentInfo>,
    pub full: Fragment,
}

impl Fragment {
    /// Constructs a new `Fragment` instance.
    pub fn new<N: ThreadSafeLayoutNode>(node: &N, specific: SpecificFragmentInfo, ctx: &LayoutContext) -> Fragment {
        let shared_context = ctx.shared_context();
        let style = node.style(shared_context);
        let writing_mode = style.writing_mode;

        let mut restyle_damage = node.restyle_damage();
        restyle_damage.remove(RECONSTRUCT_FLOW);

        Fragment {
            node: node.opaque(),
            style: style,
            selected_style: node.selected_style(),
            restyle_damage: restyle_damage,
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: specific,
            inline_context: None,
            pseudo: node.get_pseudo_element_type().strip(),
            flags: FragmentFlags::empty(),
            debug_id: DebugId::new(),
            stacking_context_id: StackingContextId::root(),
        }
    }

    /// Constructs a new `Fragment` instance from an opaque node.
    pub fn from_opaque_node_and_style(node: OpaqueNode,
                                      pseudo: PseudoElementType<()>,
                                      style: StyleArc<ServoComputedValues>,
                                      selected_style: StyleArc<ServoComputedValues>,
                                      mut restyle_damage: RestyleDamage,
                                      specific: SpecificFragmentInfo)
                                      -> Fragment {
        let writing_mode = style.writing_mode;

        restyle_damage.remove(RECONSTRUCT_FLOW);

        Fragment {
            node: node,
            style: style,
            selected_style: selected_style,
            restyle_damage: restyle_damage,
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: specific,
            inline_context: None,
            pseudo: pseudo,
            flags: FragmentFlags::empty(),
            debug_id: DebugId::new(),
            stacking_context_id: StackingContextId::root(),
        }
    }

    /// Creates an anonymous fragment just like this one but with the given style and fragment
    /// type. For the new anonymous fragment, layout-related values (border box, etc.) are reset to
    /// initial values.
    pub fn create_similar_anonymous_fragment(&self,
                                             style: StyleArc<ServoComputedValues>,
                                             specific: SpecificFragmentInfo)
                                             -> Fragment {
        let writing_mode = style.writing_mode;
        Fragment {
            node: self.node,
            style: style,
            selected_style: self.selected_style.clone(),
            restyle_damage: self.restyle_damage,
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: specific,
            inline_context: None,
            pseudo: self.pseudo,
            flags: FragmentFlags::empty(),
            debug_id: DebugId::new(),
            stacking_context_id: StackingContextId::root(),
        }
    }

    /// Transforms this fragment into another fragment of the given type, with the given size,
    /// preserving all the other data.
    pub fn transform(&self, size: LogicalSize<Au>, info: SpecificFragmentInfo)
                     -> Fragment {
        let new_border_box = LogicalRect::from_point_size(self.style.writing_mode,
                                                          self.border_box.start,
                                                          size);

        let mut restyle_damage = RestyleDamage::rebuild_and_reflow();
        restyle_damage.remove(RECONSTRUCT_FLOW);

        Fragment {
            node: self.node,
            style: self.style.clone(),
            selected_style: self.selected_style.clone(),
            restyle_damage: restyle_damage,
            border_box: new_border_box,
            border_padding: self.border_padding,
            margin: self.margin,
            specific: info,
            inline_context: self.inline_context.clone(),
            pseudo: self.pseudo.clone(),
            flags: FragmentFlags::empty(),
            debug_id: self.debug_id.clone(),
            stacking_context_id: StackingContextId::root(),
        }
    }

    /// Transforms this fragment using the given `SplitInfo`, preserving all the other data.
    pub fn transform_with_split_info(&self, split: &SplitInfo, text_run: Arc<TextRun>)
                                     -> Fragment {
        let size = LogicalSize::new(self.style.writing_mode,
                                    split.inline_size,
                                    self.border_box.size.block);
        // Preserve the insertion point if it is in this fragment's range or it is at line end.
        let (flags, insertion_point) = match self.specific {
            SpecificFragmentInfo::ScannedText(ref info) => {
                match info.insertion_point {
                    Some(index) if split.range.contains(index) => (info.flags, info.insertion_point),
                    Some(index) if index == ByteIndex(text_run.text.chars().count() as isize - 1) &&
                        index == split.range.end() => (info.flags, info.insertion_point),
                    _ => (info.flags, None)
                }
            },
            _ => (ScannedTextFlags::empty(), None)
        };
        let info = box ScannedTextFragmentInfo::new(
            text_run,
            split.range,
            size,
            insertion_point,
            flags);
        self.transform(size, SpecificFragmentInfo::ScannedText(info))
    }

    /// Transforms this fragment into an ellipsis fragment, preserving all the other data.
    pub fn transform_into_ellipsis(&self,
                                   layout_context: &LayoutContext,
                                   text_overflow_string: String)
                                   -> Fragment {
        let mut unscanned_ellipsis_fragments = LinkedList::new();
        let mut ellipsis_fragment = self.transform(
            self.border_box.size,
            SpecificFragmentInfo::UnscannedText(
                box UnscannedTextFragmentInfo::new(text_overflow_string, None)));
        unscanned_ellipsis_fragments.push_back(ellipsis_fragment);
        let ellipsis_fragments = with_thread_local_font_context(layout_context, |font_context| {
            TextRunScanner::new().scan_for_runs(font_context, unscanned_ellipsis_fragments)
        });
        debug_assert!(ellipsis_fragments.len() == 1);
        ellipsis_fragment = ellipsis_fragments.fragments.into_iter().next().unwrap();
        ellipsis_fragment.flags |= IS_ELLIPSIS;
        ellipsis_fragment
    }

    pub fn restyle_damage(&self) -> RestyleDamage {
        self.restyle_damage | self.specific.restyle_damage()
    }

    pub fn contains_node(&self, node_address: OpaqueNode) -> bool {
        node_address == self.node ||
        self.inline_context.as_ref().map_or(false, |ctx| {
            ctx.contains_node(node_address)
        })
    }

    /// Adds a style to the inline context for this fragment. If the inline context doesn't exist
    /// yet, it will be created.
    pub fn add_inline_context_style(&mut self, node_info: InlineFragmentNodeInfo) {
        if self.inline_context.is_none() {
            self.inline_context = Some(InlineFragmentContext::new());
        }
        self.inline_context.as_mut().unwrap().nodes.push(node_info);
    }

    /// Determines which quantities (border/padding/margin/specified) should be included in the
    /// intrinsic inline size of this fragment.
    fn quantities_included_in_intrinsic_inline_size(&self)
                                                    -> QuantitiesIncludedInIntrinsicInlineSizes {
        match self.specific {
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Generic |
            SpecificFragmentInfo::GeneratedContent(_) |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::InlineAbsolute(_) |
            SpecificFragmentInfo::Multicol |
            SpecificFragmentInfo::Svg(_) => {
                QuantitiesIncludedInIntrinsicInlineSizes::all()
            }
            SpecificFragmentInfo::Table => {
                INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED |
                    INTRINSIC_INLINE_SIZE_INCLUDES_PADDING |
                    INTRINSIC_INLINE_SIZE_INCLUDES_BORDER
            }
            SpecificFragmentInfo::TableCell => {
                let base_quantities = INTRINSIC_INLINE_SIZE_INCLUDES_PADDING |
                    INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED;
                if self.style.get_inheritedtable().border_collapse ==
                        border_collapse::T::separate {
                    base_quantities | INTRINSIC_INLINE_SIZE_INCLUDES_BORDER
                } else {
                    base_quantities
                }
            }
            SpecificFragmentInfo::TableWrapper => {
                let base_quantities = INTRINSIC_INLINE_SIZE_INCLUDES_MARGINS |
                    INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED;
                if self.style.get_inheritedtable().border_collapse ==
                        border_collapse::T::separate {
                    base_quantities | INTRINSIC_INLINE_SIZE_INCLUDES_BORDER
                } else {
                    base_quantities
                }
            }
            SpecificFragmentInfo::TableRow => {
                let base_quantities = INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED;
                if self.style.get_inheritedtable().border_collapse ==
                        border_collapse::T::separate {
                    base_quantities | INTRINSIC_INLINE_SIZE_INCLUDES_BORDER
                } else {
                    base_quantities
                }
            }
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::TableColumn(_) |
            SpecificFragmentInfo::UnscannedText(_) |
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) |
            SpecificFragmentInfo::InlineBlock(_) |
            SpecificFragmentInfo::MulticolColumn => {
                QuantitiesIncludedInIntrinsicInlineSizes::empty()
            }
        }
    }

    /// Returns the portion of the intrinsic inline-size that consists of borders/padding and
    /// margins, respectively.
    ///
    /// FIXME(#2261, pcwalton): This won't work well for inlines: is this OK?
    pub fn surrounding_intrinsic_inline_size(&self) -> (Au, Au) {
        let flags = self.quantities_included_in_intrinsic_inline_size();
        let style = self.style();

        // FIXME(pcwalton): Percentages should be relative to any definite size per CSS-SIZING.
        // This will likely need to be done by pushing down definite sizes during selector
        // cascading.
        let margin = if flags.contains(INTRINSIC_INLINE_SIZE_INCLUDES_MARGINS) {
            let margin = style.logical_margin();
            (MaybeAuto::from_style(margin.inline_start, Au(0)).specified_or_zero() +
             MaybeAuto::from_style(margin.inline_end, Au(0)).specified_or_zero())
        } else {
            Au(0)
        };

        // FIXME(pcwalton): Percentages should be relative to any definite size per CSS-SIZING.
        // This will likely need to be done by pushing down definite sizes during selector
        // cascading.
        let padding = if flags.contains(INTRINSIC_INLINE_SIZE_INCLUDES_PADDING) {
            let padding = style.logical_padding();
            (model::specified(padding.inline_start, Au(0)) +
             model::specified(padding.inline_end, Au(0)))
        } else {
            Au(0)
        };

        let border = if flags.contains(INTRINSIC_INLINE_SIZE_INCLUDES_BORDER) {
            self.border_width().inline_start_end()
        } else {
            Au(0)
        };

        (border + padding, margin)
    }

    /// Uses the style only to estimate the intrinsic inline-sizes. These may be modified for text
    /// or replaced elements.
    pub fn style_specified_intrinsic_inline_size(&self) -> IntrinsicISizesContribution {
        let flags = self.quantities_included_in_intrinsic_inline_size();
        let style = self.style();

        // FIXME(#2261, pcwalton): This won't work well for inlines: is this OK?
        let (border_padding, margin) = self.surrounding_intrinsic_inline_size();

        let mut specified = Au(0);
        if flags.contains(INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED) {
            specified = MaybeAuto::from_style(style.content_inline_size(),
                                              Au(0)).specified_or_zero();
            specified = max(model::specified(style.min_inline_size(), Au(0)), specified);
            if let Some(max) = model::specified_or_none(style.max_inline_size(), Au(0)) {
                specified = min(specified, max)
            }

            if self.style.get_position().box_sizing == box_sizing::T::border_box {
                specified = max(Au(0), specified - border_padding);
            }
        }

        IntrinsicISizesContribution {
            content_intrinsic_sizes: IntrinsicISizes {
                minimum_inline_size: specified,
                preferred_inline_size: specified,
            },
            surrounding_size: border_padding + margin,
        }
    }

    /// intrinsic width of this replaced element.
    #[inline]
    pub fn intrinsic_width(&self) -> Au {
        match self.specific {
            SpecificFragmentInfo::Image(ref info) => {
                if let Some(ref data) = info.metadata {
                    Au::from_px(data.width as i32)
                } else {
                    Au(0)
                }
            }
            SpecificFragmentInfo::Canvas(ref info) => info.dom_width,
            SpecificFragmentInfo::Svg(ref info) => info.dom_width,
            // Note: Currently for replaced element with no intrinsic size,
            // this function simply returns the default object size. As long as
            // these elements do not have intrinsic aspect ratio this should be
            // sufficient, but we may need to investigate if this is enough for
            // use cases like SVG.
            SpecificFragmentInfo::Iframe(_) => Au::from_px(DEFAULT_REPLACED_WIDTH),
            _ => panic!("Trying to get intrinsic width on non-replaced element!")
        }
    }

    /// intrinsic width of this replaced element.
    #[inline]
    pub fn intrinsic_height(&self) -> Au {
        match self.specific {
            SpecificFragmentInfo::Image(ref info) => {
                if let Some(ref data) = info.metadata {
                    Au::from_px(data.height as i32)
                } else {
                    Au(0)
                }
            }
            SpecificFragmentInfo::Canvas(ref info) => info.dom_height,
            SpecificFragmentInfo::Svg(ref info) => info.dom_height,
            SpecificFragmentInfo::Iframe(_) => Au::from_px(DEFAULT_REPLACED_HEIGHT),
            _ => panic!("Trying to get intrinsic height on non-replaced element!")
        }
    }

    /// Whether this replace element has intrinsic aspect ratio.
    pub fn has_intrinsic_ratio(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::Image(_)  |
            SpecificFragmentInfo::Canvas(_) |
            // TODO(stshine): According to the SVG spec, whether a SVG element has intrinsic
            // aspect ratio is determined by the `preserveAspectRatio` attribute. Since for
            // now SVG is far from implemented, we simply choose the default behavior that
            // the intrinsic aspect ratio is preserved.
            // https://svgwg.org/svg2-draft/coords.html#PreserveAspectRatioAttribute
            SpecificFragmentInfo::Svg(_) =>
                self.intrinsic_width() != Au(0) && self.intrinsic_height() != Au(0),
            _ => false
        }
    }

    /// CSS 2.1 § 10.3.2 & 10.6.2 Calculate the used width and height of a replaced element.
    /// When a parameter is `None` it means the specified size in certain direction
    /// is unconstrained. The inline containing size can also be `None` since this
    /// method is also used for calculating intrinsic inline size contribution.
    pub fn calculate_replaced_sizes(&self,
                                    containing_inline_size: Option<Au>,
                                    containing_block_size: Option<Au>)
                                    -> (Au, Au) {
        let (intrinsic_inline_size, intrinsic_block_size) = if self.style.writing_mode.is_vertical() {
            (self.intrinsic_height(), self.intrinsic_width())
        } else {
            (self.intrinsic_width(), self.intrinsic_height())
        };

        // Make sure the size we used here is for content box since they may be
        // transferred by the intrinsic aspect ratio.
        let inline_size = style_length(self.style.content_inline_size(), containing_inline_size)
                                     .map(|x| x - self.box_sizing_boundary(Direction::Inline));
        let block_size = style_length(self.style.content_block_size(), containing_block_size)
                                     .map(|x| x - self.box_sizing_boundary(Direction::Block));
        let inline_constraint = self.size_constraint(containing_inline_size, Direction::Inline);
        let block_constraint = self.size_constraint(containing_block_size, Direction::Block);

        // https://drafts.csswg.org/css-images-3/#default-sizing
        match (inline_size, block_size) {
            // If the specified size is a definite width and height, the concrete
            // object size is given that width and height.
            (MaybeAuto::Specified(inline_size), MaybeAuto::Specified(block_size)) =>
                (inline_constraint.clamp(inline_size), block_constraint.clamp(block_size)),

            // If the specified size is only a width or height (but not both)
            // then the concrete object size is given that specified width or
            // height. The other dimension is calculated as follows:
            //
            // If the object has an intrinsic aspect ratio, the missing dimension
            // of the concrete object size is calculated using the intrinsic
            // aspect ratio and the present dimension.
            //
            // Otherwise, if the missing dimension is present in the object’s intrinsic
            // dimensions, the missing dimension is taken from the object’s intrinsic
            // dimensions. Otherwise it is taken from the default object size.
            (MaybeAuto::Specified(inline_size), MaybeAuto::Auto) => {
                let inline_size = inline_constraint.clamp(inline_size);
                let block_size = if self.has_intrinsic_ratio() {
                    // Note: We can not precompute the ratio and store it as a float, because
                    // doing so may result one pixel difference in calculation for certain
                    // images, thus make some tests fail.
                    Au((inline_size.0 as i64 * intrinsic_block_size.0 as i64 /
                        intrinsic_inline_size.0 as i64) as i32)
                } else {
                    intrinsic_block_size
                };
                (inline_size, block_constraint.clamp(block_size))
            }
            (MaybeAuto::Auto, MaybeAuto::Specified(block_size)) => {
                let block_size = block_constraint.clamp(block_size);
                let inline_size = if self.has_intrinsic_ratio() {
                    Au((block_size.0 as i64 * intrinsic_inline_size.0 as i64 /
                       intrinsic_block_size.0 as i64) as i32)
                } else {
                    intrinsic_inline_size
                };
                (inline_constraint.clamp(inline_size), block_size)
            }
            // https://drafts.csswg.org/css2/visudet.html#min-max-widths
            (MaybeAuto::Auto, MaybeAuto::Auto) => {
                if self.has_intrinsic_ratio() {
                    // This approch follows the spirit of cover and contain constraint.
                    // https://drafts.csswg.org/css-images-3/#cover-contain

                    // First, create two rectangles that keep aspect ratio while may be clamped
                    // by the contraints;
                    let first_isize = inline_constraint.clamp(intrinsic_inline_size);
                    let first_bsize = Au((first_isize.0 as i64 * intrinsic_block_size.0 as i64 /
                                          intrinsic_inline_size.0 as i64) as i32);
                    let second_bsize = block_constraint.clamp(intrinsic_block_size);
                    let second_isize = Au((second_bsize.0 as i64 * intrinsic_inline_size.0 as i64 /
                                           intrinsic_block_size.0 as i64) as i32);
                    let (inline_size, block_size) = match (first_isize.cmp(&intrinsic_inline_size) ,
                                                           second_isize.cmp(&intrinsic_inline_size)) {
                        (Ordering::Equal, Ordering::Equal) =>
                            (first_isize, first_bsize),
                        // When only one rectangle is clamped, use it;
                        (Ordering::Equal, _) =>
                            (second_isize, second_bsize),
                        (_, Ordering::Equal) =>
                            (first_isize, first_bsize),
                        // When both rectangles grow (smaller than min sizes),
                        // Choose the larger one;
                        (Ordering::Greater, Ordering::Greater) =>
                            if first_isize > second_isize {
                                (first_isize, first_bsize)
                            } else {
                                (second_isize, second_bsize)
                            },
                        // When both rectangles shrink (larger than max sizes),
                        // Choose the smaller one;
                        (Ordering::Less, Ordering::Less) =>
                            if first_isize > second_isize {
                                (second_isize, second_bsize)
                            } else {
                                (first_isize, first_bsize)
                            },
                        // It does not matter which we choose here, because both sizes
                        // will be clamped to constraint;
                        (Ordering::Less, Ordering::Greater) | (Ordering::Greater, Ordering::Less) =>
                            (first_isize, first_bsize)
                    };
                    // Clamp the result and we are done :-)
                    (inline_constraint.clamp(inline_size), block_constraint.clamp(block_size))
                } else {
                    (inline_constraint.clamp(intrinsic_inline_size),
                    block_constraint.clamp(intrinsic_block_size))
                }
            }
        }
    }

    /// Return a size constraint that can be used the clamp size in given direction.
    /// To take `box-sizing: border-box` into account, the `border_padding` field
    /// must be initialized first.
    ///
    /// TODO(stshine): Maybe there is a more convenient way.
    pub fn size_constraint(&self, containing_size: Option<Au>, direction: Direction) -> SizeConstraint {
        let (style_min_size, style_max_size) = match direction {
            Direction::Inline => (self.style.min_inline_size(), self.style.max_inline_size()),
            Direction::Block => (self.style.min_block_size(), self.style.max_block_size())
        };

        let border = if self.style().get_position().box_sizing == box_sizing::T::border_box {
            Some(self.border_padding.start_end(direction))
        } else {
            None
        };

        SizeConstraint::new(containing_size, style_min_size, style_max_size, border)
    }

    /// Returns a guess as to the distances from the margin edge of this fragment to its content
    /// in the inline direction. This will generally be correct unless percentages are involved.
    ///
    /// This is used for the float placement speculation logic.
    pub fn guess_inline_content_edge_offsets(&self) -> SpeculatedInlineContentEdgeOffsets {
        let logical_margin = self.style.logical_margin();
        let logical_padding = self.style.logical_padding();
        let border_width = self.border_width();
        SpeculatedInlineContentEdgeOffsets {
            start: MaybeAuto::from_style(logical_margin.inline_start, Au(0)).specified_or_zero() +
                model::specified(logical_padding.inline_start, Au(0)) +
                border_width.inline_start,
            end: MaybeAuto::from_style(logical_margin.inline_end, Au(0)).specified_or_zero() +
                model::specified(logical_padding.inline_end, Au(0)) +
                border_width.inline_end,
        }
    }

    /// Returns the sum of the inline-sizes of all the borders of this fragment. Note that this
    /// can be expensive to compute, so if possible use the `border_padding` field instead.
    #[inline]
    pub fn border_width(&self) -> LogicalMargin<Au> {
        let style_border_width = self.style().logical_border_width();

        // NOTE: We can have nodes with different writing mode inside
        // the inline fragment context, so we need to overwrite the
        // writing mode to compute the child logical sizes.
        let writing_mode = self.style.writing_mode;
        let context_border = match self.inline_context {
            None => LogicalMargin::zero(writing_mode),
            Some(ref inline_fragment_context) => {
                inline_fragment_context.nodes.iter().fold(style_border_width, |accumulator, node| {
                    let mut this_border_width =
                        node.style.border_width_for_writing_mode(writing_mode);
                    if !node.flags.contains(FIRST_FRAGMENT_OF_ELEMENT) {
                        this_border_width.inline_start = Au(0)
                    }
                    if !node.flags.contains(LAST_FRAGMENT_OF_ELEMENT) {
                        this_border_width.inline_end = Au(0)
                    }
                    accumulator + this_border_width
                })
            }
        };
        style_border_width + context_border
    }

    /// Returns the border width in given direction if this fragment has property
    /// 'box-sizing: border-box'. The `border_padding` field must have been initialized.
    pub fn box_sizing_boundary(&self, direction: Direction) -> Au {
        match (self.style().get_position().box_sizing, direction) {
            (box_sizing::T::border_box, Direction::Inline) => {
                self.border_padding.inline_start_end()
            }
            (box_sizing::T::border_box, Direction::Block) => {
                self.border_padding.block_start_end()
            }
            _ => Au(0)
        }
    }

    /// Computes the margins in the inline direction from the containing block inline-size and the
    /// style. After this call, the inline direction of the `margin` field will be correct.
    ///
    /// Do not use this method if the inline direction margins are to be computed some other way
    /// (for example, via constraint solving for blocks).
    pub fn compute_inline_direction_margins(&mut self, containing_block_inline_size: Au) {
        match self.specific {
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableColumn(_) |
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) => {
                self.margin.inline_start = Au(0);
                self.margin.inline_end = Au(0);
                return
            }
            _ => {
                let margin = self.style().logical_margin();
                self.margin.inline_start =
                    MaybeAuto::from_style(margin.inline_start,
                                          containing_block_inline_size).specified_or_zero();
                self.margin.inline_end =
                    MaybeAuto::from_style(margin.inline_end,
                                          containing_block_inline_size).specified_or_zero();
            }
        }

        if let Some(ref inline_context) = self.inline_context {
            for node in &inline_context.nodes {
                let margin = node.style.logical_margin();
                let this_inline_start_margin = if !node.flags.contains(FIRST_FRAGMENT_OF_ELEMENT) {
                    Au(0)
                } else {
                    MaybeAuto::from_style(margin.inline_start,
                                          containing_block_inline_size).specified_or_zero()
                };
                let this_inline_end_margin = if !node.flags.contains(LAST_FRAGMENT_OF_ELEMENT) {
                    Au(0)
                } else {
                    MaybeAuto::from_style(margin.inline_end,
                                          containing_block_inline_size).specified_or_zero()
                };

                self.margin.inline_start += this_inline_start_margin;
                self.margin.inline_end += this_inline_end_margin;
            }
        }
    }

    /// Computes the margins in the block direction from the containing block inline-size and the
    /// style. After this call, the block direction of the `margin` field will be correct.
    ///
    /// Do not use this method if the block direction margins are to be computed some other way
    /// (for example, via constraint solving for absolutely-positioned flows).
    pub fn compute_block_direction_margins(&mut self, containing_block_inline_size: Au) {
        match self.specific {
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableColumn(_) => {
                self.margin.block_start = Au(0);
                self.margin.block_end = Au(0)
            }
            _ => {
                // NB: Percentages are relative to containing block inline-size (not block-size)
                // per CSS 2.1.
                let margin = self.style().logical_margin();
                self.margin.block_start =
                    MaybeAuto::from_style(margin.block_start, containing_block_inline_size)
                    .specified_or_zero();
                self.margin.block_end =
                    MaybeAuto::from_style(margin.block_end, containing_block_inline_size)
                    .specified_or_zero();
            }
        }
    }

    /// Computes the border and padding in both inline and block directions from the containing
    /// block inline-size and the style. After this call, the `border_padding` field will be
    /// correct.
    ///
    /// TODO(pcwalton): Remove `border_collapse`; we can figure it out from our style and specific
    /// fragment info.
    pub fn compute_border_and_padding(&mut self,
                                      containing_block_inline_size: Au,
                                      border_collapse: border_collapse::T) {
        // Compute border.
        let border = match border_collapse {
            border_collapse::T::separate => self.border_width(),
            border_collapse::T::collapse => LogicalMargin::zero(self.style.writing_mode),
        };

        // Compute padding from the fragment's style.
        let padding_from_style = match self.specific {
            SpecificFragmentInfo::TableColumn(_) |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableWrapper => LogicalMargin::zero(self.style.writing_mode),
            _ => model::padding_from_style(self.style(), containing_block_inline_size, self.style().writing_mode),
        };

        // Compute padding from the inline fragment context.
        let padding_from_inline_fragment_context = match (&self.specific, &self.inline_context) {
            (_, &None) |
            (&SpecificFragmentInfo::TableColumn(_), _) |
            (&SpecificFragmentInfo::TableRow, _) |
            (&SpecificFragmentInfo::TableWrapper, _) => {
                LogicalMargin::zero(self.style.writing_mode)
            }
            (_, &Some(ref inline_fragment_context)) => {
                let writing_mode = self.style.writing_mode;
                let zero_padding = LogicalMargin::zero(writing_mode);
                inline_fragment_context.nodes.iter().fold(zero_padding, |accumulator, node| {
                    let mut padding = model::padding_from_style(&*node.style, Au(0), writing_mode);
                    if !node.flags.contains(FIRST_FRAGMENT_OF_ELEMENT) {
                        padding.inline_start = Au(0)
                    }
                    if !node.flags.contains(LAST_FRAGMENT_OF_ELEMENT) {
                        padding.inline_end = Au(0)
                    }
                    accumulator + padding
                })
            }
        };

        self.border_padding = border + padding_from_style + padding_from_inline_fragment_context
    }

    // Return offset from original position because of `position: relative`.
    pub fn relative_position(&self, containing_block_size: &LogicalSize<Au>) -> LogicalSize<Au> {
        fn from_style(style: &ServoComputedValues, container_size: &LogicalSize<Au>)
                      -> LogicalSize<Au> {
            let offsets = style.logical_position();
            let offset_i = if offsets.inline_start != LengthOrPercentageOrAuto::Auto {
                MaybeAuto::from_style(offsets.inline_start,
                                      container_size.inline).specified_or_zero()
            } else {
                -MaybeAuto::from_style(offsets.inline_end,
                                       container_size.inline).specified_or_zero()
            };
            let offset_b = if offsets.block_start != LengthOrPercentageOrAuto::Auto {
                MaybeAuto::from_style(offsets.block_start,
                                      container_size.block).specified_or_zero()
            } else {
                -MaybeAuto::from_style(offsets.block_end,
                                       container_size.block).specified_or_zero()
            };
            LogicalSize::new(style.writing_mode, offset_i, offset_b)
        }

        // Go over the ancestor fragments and add all relative offsets (if any).
        let mut rel_pos = if self.style().get_box().position == position::T::relative {
            from_style(self.style(), containing_block_size)
        } else {
            LogicalSize::zero(self.style.writing_mode)
        };

        if let Some(ref inline_fragment_context) = self.inline_context {
            for node in &inline_fragment_context.nodes {
                if node.style.get_box().position == position::T::relative {
                    rel_pos = rel_pos + from_style(&*node.style, containing_block_size);
                }
            }
        }

        rel_pos
    }

    /// Always inline for SCCP.
    ///
    /// FIXME(pcwalton): Just replace with the clear type from the style module for speed?
    #[inline(always)]
    pub fn clear(&self) -> Option<ClearType> {
        let style = self.style();
        match style.get_box().clear {
            clear::T::none => None,
            clear::T::left => Some(ClearType::Left),
            clear::T::right => Some(ClearType::Right),
            clear::T::both => Some(ClearType::Both),
        }
    }

    #[inline(always)]
    pub fn style(&self) -> &ServoComputedValues {
        &*self.style
    }

    #[inline(always)]
    pub fn selected_style(&self) -> &ServoComputedValues {
        &*self.selected_style
    }

    pub fn white_space(&self) -> white_space::T {
        self.style().get_inheritedtext().white_space
    }

    pub fn color(&self) -> color::T {
        self.style().get_color().color
    }

    /// Returns the text decoration line of this fragment, according to the style of the nearest ancestor
    /// element.
    ///
    /// NB: This may not be the actual text decoration line, because of the override rules specified in
    /// CSS 2.1 § 16.3.1. Unfortunately, computing this properly doesn't really fit into Servo's
    /// model. Therefore, this is a best lower bound approximation, but the end result may actually
    /// have the various decoration flags turned on afterward.
    pub fn text_decoration_line(&self) -> text_decoration_line::T {
        self.style().get_text().text_decoration_line
    }

    /// Returns the inline-start offset from margin edge to content edge.
    ///
    /// FIXME(#2262, pcwalton): I think this method is pretty bogus, because it won't work for
    /// inlines.
    pub fn inline_start_offset(&self) -> Au {
        match self.specific {
            SpecificFragmentInfo::TableWrapper => self.margin.inline_start,
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableRow => self.border_padding.inline_start,
            SpecificFragmentInfo::TableColumn(_) => Au(0),
            _ => self.margin.inline_start + self.border_padding.inline_start,
        }
    }

    /// Returns true if this element can be split. This is true for text fragments, unless
    /// `white-space: pre` or `white-space: nowrap` is set.
    pub fn can_split(&self) -> bool {
        self.is_scanned_text_fragment() && self.white_space().allow_wrap()
    }

    /// Returns true if and only if this fragment is a generated content fragment.
    pub fn is_unscanned_generated_content(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::GeneratedContent(box GeneratedContentInfo::Empty) => false,
            SpecificFragmentInfo::GeneratedContent(..) => true,
            _ => false,
        }
    }

    /// Returns true if and only if this is a scanned text fragment.
    pub fn is_scanned_text_fragment(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::ScannedText(..) => true,
            _ => false,
        }
    }

    /// Computes the intrinsic inline-sizes of this fragment.
    pub fn compute_intrinsic_inline_sizes(&mut self) -> IntrinsicISizesContribution {
        let mut result = self.style_specified_intrinsic_inline_size();
        match self.specific {
            SpecificFragmentInfo::Generic |
            SpecificFragmentInfo::GeneratedContent(_) |
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableColumn(_) |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableWrapper |
            SpecificFragmentInfo::Multicol |
            SpecificFragmentInfo::MulticolColumn |
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) => {}
            SpecificFragmentInfo::InlineBlock(ref info) => {
                let block_flow = info.flow_ref.as_block();
                result.union_block(&block_flow.base.intrinsic_inline_sizes)
            }
            SpecificFragmentInfo::InlineAbsolute(ref info) => {
                let block_flow = info.flow_ref.as_block();
                result.union_block(&block_flow.base.intrinsic_inline_sizes)
            }
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Svg(_) => {
                let mut inline_size = match self.style.content_inline_size() {
                    LengthOrPercentageOrAuto::Auto |
                    LengthOrPercentageOrAuto::Percentage(_) => {
                        // We have to initialize the `border_padding` field first to make
                        // the size constraints work properly.
                        // TODO(stshine): Find a cleaner way to do this.
                        let padding = self.style.logical_padding();
                        self.border_padding.inline_start = model::specified(padding.inline_start, Au(0));
                        self.border_padding.inline_end = model::specified(padding.inline_end, Au(0));
                        self.border_padding.block_start = model::specified(padding.block_start, Au(0));
                        self.border_padding.block_end = model::specified(padding.block_end, Au(0));
                        let border = self.border_width();
                        self.border_padding.inline_start += border.inline_start;
                        self.border_padding.inline_end += border.inline_end;
                        self.border_padding.block_start += border.block_start;
                        self.border_padding.block_end += border.block_end;
                        let (result_inline, _) = self.calculate_replaced_sizes(None, None);
                        result_inline
                    }
                    LengthOrPercentageOrAuto::Length(length) => length,
                    LengthOrPercentageOrAuto::Calc(calc) => calc.length(),
                };

                let size_constraint = self.size_constraint(None, Direction::Inline);
                inline_size = size_constraint.clamp(inline_size);

                result.union_block(&IntrinsicISizes {
                    minimum_inline_size: inline_size,
                    preferred_inline_size: inline_size,
                });
            }

            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: Some(ref text_fragment_info),
                ..
            }) |
            SpecificFragmentInfo::ScannedText(box ref text_fragment_info) => {
                let range = &text_fragment_info.range;

                // See http://dev.w3.org/csswg/css-sizing/#max-content-inline-size.
                // TODO: Account for soft wrap opportunities.
                let max_line_inline_size = text_fragment_info.run
                                                             .metrics_for_range(range)
                                                             .advance_width;

                let min_line_inline_size = if self.white_space().allow_wrap() {
                    text_fragment_info.run.min_width_for_range(range)
                } else {
                    max_line_inline_size
                };

                result.union_block(&IntrinsicISizes {
                    minimum_inline_size: min_line_inline_size,
                    preferred_inline_size: max_line_inline_size,
                })
            }

            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: None,
                ..
            }) => return IntrinsicISizesContribution::new(),

            SpecificFragmentInfo::UnscannedText(..) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            }
        };

        // Take borders and padding for parent inline fragments into account.
        let writing_mode = self.style.writing_mode;
        if let Some(ref context) = self.inline_context {
            for node in &context.nodes {
                let mut border_width = node.style.logical_border_width();
                let mut padding = model::padding_from_style(&*node.style, Au(0), writing_mode);
                let mut margin = model::specified_margin_from_style(&*node.style, writing_mode);
                if !node.flags.contains(FIRST_FRAGMENT_OF_ELEMENT) {
                    border_width.inline_start = Au(0);
                    padding.inline_start = Au(0);
                    margin.inline_start = Au(0);
                }
                if !node.flags.contains(LAST_FRAGMENT_OF_ELEMENT) {
                    border_width.inline_end = Au(0);
                    padding.inline_end = Au(0);
                    margin.inline_end = Au(0);
                }

                result.surrounding_size =
                    result.surrounding_size +
                    border_width.inline_start_end() +
                    padding.inline_start_end() +
                    margin.inline_start_end();
            }
        }

        result
    }

    /// Returns the narrowest inline-size that the first splittable part of this fragment could
    /// possibly be split to. (In most cases, this returns the inline-size of the first word in
    /// this fragment.)
    pub fn minimum_splittable_inline_size(&self) -> Au {
        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: Some(ref text),
                ..
            }) |
            SpecificFragmentInfo::ScannedText(box ref text) => {
                text.run.minimum_splittable_inline_size(&text.range)
            }
            _ => Au(0),
        }
    }

    /// Returns the dimensions of the content box.
    ///
    /// This is marked `#[inline]` because it is frequently called when only one or two of the
    /// values are needed and that will save computation.
    #[inline]
    pub fn content_box(&self) -> LogicalRect<Au> {
        self.border_box - self.border_padding
    }

    /// Attempts to find the split positions of a text fragment so that its inline-size is no more
    /// than `max_inline_size`.
    ///
    /// A return value of `None` indicates that the fragment could not be split. Otherwise the
    /// information pertaining to the split is returned. The inline-start and inline-end split
    /// information are both optional due to the possibility of them being whitespace.
    pub fn calculate_split_position(&self, max_inline_size: Au, starts_line: bool)
                                    -> Option<SplitResult> {
        let text_fragment_info = match self.specific {
            SpecificFragmentInfo::ScannedText(ref text_fragment_info)
                => text_fragment_info,
            _   => return None,
        };

        let mut flags = SplitOptions::empty();
        if starts_line {
            flags.insert(STARTS_LINE);
            if self.style().get_inheritedtext().overflow_wrap == overflow_wrap::T::break_word {
                flags.insert(RETRY_AT_CHARACTER_BOUNDARIES)
            }
        }

        match self.style().get_inheritedtext().word_break {
            word_break::T::normal | word_break::T::keep_all => {
                // Break at normal word boundaries. keep-all forbids soft wrap opportunities.
                let natural_word_breaking_strategy =
                    text_fragment_info.run.natural_word_slices_in_range(&text_fragment_info.range);
                self.calculate_split_position_using_breaking_strategy(
                    natural_word_breaking_strategy,
                    max_inline_size,
                    flags)
            }
            word_break::T::break_all => {
                // Break at character boundaries.
                let character_breaking_strategy =
                    text_fragment_info.run.character_slices_in_range(&text_fragment_info.range);
                flags.remove(RETRY_AT_CHARACTER_BOUNDARIES);
                self.calculate_split_position_using_breaking_strategy(
                    character_breaking_strategy,
                    max_inline_size,
                    flags)
            }
        }
    }

    /// Truncates this fragment to the given `max_inline_size`, using a character-based breaking
    /// strategy. The resulting fragment will have `SpecificFragmentInfo::TruncatedFragment`,
    /// preserving the original fragment for use in incremental reflow.
    ///
    /// This function will panic if self is already truncated.
    pub fn truncate_to_inline_size(self, max_inline_size: Au) -> Fragment {
        if let SpecificFragmentInfo::TruncatedFragment(_) = self.specific {
            panic!("Cannot truncate an already truncated fragment");
        }
        let info = self.calculate_truncate_to_inline_size(max_inline_size);
        let (size, text_info) = match info {
            Some(TruncationResult { split: SplitInfo { inline_size, range }, text_run } ) => {
                let size = LogicalSize::new(self.style.writing_mode,
                                            inline_size,
                                            self.border_box.size.block);
                // Preserve the insertion point if it is in this fragment's range or it is at line end.
                let (flags, insertion_point) = match self.specific {
                    SpecificFragmentInfo::ScannedText(ref info) => {
                        match info.insertion_point {
                            Some(index) if range.contains(index) => (info.flags, info.insertion_point),
                            Some(index) if index == ByteIndex(text_run.text.chars().count() as isize - 1) &&
                                index == range.end() => (info.flags, info.insertion_point),
                            _ => (info.flags, None)
                        }
                    },
                    _ => (ScannedTextFlags::empty(), None)
                };
                let text_info = ScannedTextFragmentInfo::new(
                    text_run,
                    range,
                    size,
                    insertion_point,
                    flags);
                (size, Some(text_info))
            }
            None =>
                (LogicalSize::zero(self.style.writing_mode), None)
        };
        let mut result = self.transform(size, SpecificFragmentInfo::Generic);
        result.specific = SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
            text_info: text_info,
            full: self,
        });
        result
    }

    /// Truncates this fragment to the given `max_inline_size`, using a character-based breaking
    /// strategy. If no characters could fit, returns `None`.
    fn calculate_truncate_to_inline_size(&self, max_inline_size: Au) -> Option<TruncationResult> {
        let text_fragment_info =
            if let SpecificFragmentInfo::ScannedText(ref text_fragment_info) = self.specific {
                text_fragment_info
            } else {
                return None
            };

        let character_breaking_strategy =
            text_fragment_info.run.character_slices_in_range(&text_fragment_info.range);
        match self.calculate_split_position_using_breaking_strategy(character_breaking_strategy,
                                                                    max_inline_size,
                                                                    SplitOptions::empty()) {
            None => None,
            Some(split_info) => {
                match split_info.inline_start {
                    None => None,
                    Some(split) => {
                        Some(TruncationResult {
                            split: split,
                            text_run: split_info.text_run.clone(),
                        })
                    }
                }
            }
        }
    }

    /// A helper method that uses the breaking strategy described by `slice_iterator` (at present,
    /// either natural word breaking or character breaking) to split this fragment.
    fn calculate_split_position_using_breaking_strategy<'a, I>(
            &self,
            slice_iterator: I,
            max_inline_size: Au,
            flags: SplitOptions)
            -> Option<SplitResult>
            where I: Iterator<Item=TextRunSlice<'a>> {
        let text_fragment_info = match self.specific {
            SpecificFragmentInfo::ScannedText(ref text_fragment_info)
                => text_fragment_info,
            _   => return None,
        };

        let mut remaining_inline_size = max_inline_size - self.border_padding.inline_start_end();
        let mut inline_start_range = Range::new(text_fragment_info.range.begin(), ByteIndex(0));
        let mut inline_end_range = None;
        let mut overflowing = false;

        debug!("calculate_split_position_using_breaking_strategy: splitting text fragment \
                (strlen={}, range={:?}, max_inline_size={:?})",
               text_fragment_info.run.text.len(),
               text_fragment_info.range,
               max_inline_size);

        for slice in slice_iterator {
            debug!("calculate_split_position_using_breaking_strategy: considering slice \
                    (offset={:?}, slice range={:?}, remaining_inline_size={:?})",
                   slice.offset,
                   slice.range,
                   remaining_inline_size);

            // Use the `remaining_inline_size` to find a split point if possible. If not, go around
            // the loop again with the next slice.
            let metrics = text_fragment_info.run.metrics_for_slice(slice.glyphs, &slice.range);
            let advance = metrics.advance_width;

            // Have we found the split point?
            if advance <= remaining_inline_size || slice.glyphs.is_whitespace() {
                // Keep going; we haven't found the split point yet.
                debug!("calculate_split_position_using_breaking_strategy: enlarging span");
                remaining_inline_size = remaining_inline_size - advance;
                inline_start_range.extend_by(slice.range.length());
                continue
            }

            // The advance is more than the remaining inline-size, so split here. First, check to
            // see if we're going to overflow the line. If so, perform a best-effort split.
            let mut remaining_range = slice.text_run_range();
            let split_is_empty = inline_start_range.is_empty() &&
                    !(self.requires_line_break_afterward_if_wrapping_on_newlines() &&
                      !self.white_space().allow_wrap());
            if split_is_empty {
                // We're going to overflow the line.
                overflowing = true;
                inline_start_range = slice.text_run_range();
                remaining_range = Range::new(slice.text_run_range().end(), ByteIndex(0));
                remaining_range.extend_to(text_fragment_info.range.end());
            }

            // Check to see if we need to create an inline-end chunk.
            let slice_begin = remaining_range.begin();
            if slice_begin < text_fragment_info.range.end() {
                // There still some things left over at the end of the line, so create the
                // inline-end chunk.
                let mut inline_end = remaining_range;
                inline_end.extend_to(text_fragment_info.range.end());
                inline_end_range = Some(inline_end);
                debug!("calculate_split_position: splitting remainder with inline-end range={:?}",
                       inline_end);
            }

            // If we failed to find a suitable split point, we're on the verge of overflowing the
            // line.
            if split_is_empty || overflowing {
                // If we've been instructed to retry at character boundaries (probably via
                // `overflow-wrap: break-word`), do so.
                if flags.contains(RETRY_AT_CHARACTER_BOUNDARIES) {
                    let character_breaking_strategy =
                        text_fragment_info.run
                                          .character_slices_in_range(&text_fragment_info.range);
                    let mut flags = flags;
                    flags.remove(RETRY_AT_CHARACTER_BOUNDARIES);
                    return self.calculate_split_position_using_breaking_strategy(
                        character_breaking_strategy,
                        max_inline_size,
                        flags)
                }

                // We aren't at the start of the line, so don't overflow. Let inline layout wrap to
                // the next line instead.
                if !flags.contains(STARTS_LINE) {
                    return None
                }
            }

            break
        }

        let split_is_empty = inline_start_range.is_empty() &&
                !self.requires_line_break_afterward_if_wrapping_on_newlines();
        let inline_start = if !split_is_empty {
            Some(SplitInfo::new(inline_start_range, &**text_fragment_info))
        } else {
            None
        };
        let inline_end = inline_end_range.map(|inline_end_range| {
            SplitInfo::new(inline_end_range, &**text_fragment_info)
        });

        Some(SplitResult {
            inline_start: inline_start,
            inline_end: inline_end,
            text_run: text_fragment_info.run.clone(),
        })
    }

    /// The opposite of `calculate_split_position_using_breaking_strategy`: merges this fragment
    /// with the next one.
    pub fn merge_with(&mut self, next_fragment: Fragment) {
        match (&mut self.specific, &next_fragment.specific) {
            (&mut SpecificFragmentInfo::ScannedText(ref mut this_info),
             &SpecificFragmentInfo::ScannedText(ref other_info)) => {
                debug_assert!(::arc_ptr_eq(&this_info.run, &other_info.run));
                this_info.range_end_including_stripped_whitespace =
                    other_info.range_end_including_stripped_whitespace;
                if other_info.requires_line_break_afterward_if_wrapping_on_newlines() {
                    this_info.flags.insert(REQUIRES_LINE_BREAK_AFTERWARD_IF_WRAPPING_ON_NEWLINES);
                }
                if other_info.insertion_point.is_some() {
                    this_info.insertion_point = other_info.insertion_point;
                }
                self.border_padding.inline_end = next_fragment.border_padding.inline_end;
                self.margin.inline_end = next_fragment.margin.inline_end;
            }
            _ => panic!("Can only merge two scanned-text fragments!"),
        }
        self.reset_text_range_and_inline_size();
        self.meld_with_next_inline_fragment(&next_fragment);
    }

    /// Restore any whitespace that was stripped from a text fragment, and recompute inline metrics
    /// if necessary.
    pub fn reset_text_range_and_inline_size(&mut self) {
        if let SpecificFragmentInfo::ScannedText(ref mut info) = self.specific {
            if info.run.extra_word_spacing != Au(0) {
                Arc::make_mut(&mut info.run).extra_word_spacing = Au(0);
            }

            // FIXME (mbrubeck): Do we need to restore leading too?
            let range_end = info.range_end_including_stripped_whitespace;
            if info.range.end() == range_end {
                return
            }
            info.range.extend_to(range_end);
            info.content_size.inline = info.run.metrics_for_range(&info.range).advance_width;
            self.border_box.size.inline = info.content_size.inline +
                self.border_padding.inline_start_end();
        }
    }

    /// Assigns replaced inline-size, padding, and margins for this fragment only if it is replaced
    /// content per CSS 2.1 § 10.3.2.
    pub fn assign_replaced_inline_size_if_necessary(&mut self,
                                                    container_inline_size: Au,
                                                    container_block_size: Option<Au>) {
        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: None,
                ..
            }) |
            SpecificFragmentInfo::Generic |
            SpecificFragmentInfo::GeneratedContent(_) |
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableWrapper |
            SpecificFragmentInfo::Multicol |
            SpecificFragmentInfo::MulticolColumn => return,
            SpecificFragmentInfo::TableColumn(_) => {
                panic!("Table column fragments do not have inline size")
            }
            SpecificFragmentInfo::UnscannedText(_) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            }
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::InlineBlock(_) |
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) |
            SpecificFragmentInfo::InlineAbsolute(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::Svg(_) => {}
        };

        match self.specific {
            // Inline blocks
            SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut info) => {
                let block_flow = FlowRef::deref_mut(&mut info.flow_ref).as_mut_block();
                block_flow.base.position.size.inline =
                    block_flow.base.intrinsic_inline_sizes.preferred_inline_size;

                // This is a hypothetical box, so it takes up no space.
                self.border_box.size.inline = Au(0);
            }
            SpecificFragmentInfo::InlineBlock(ref mut info) => {
                let block_flow = FlowRef::deref_mut(&mut info.flow_ref).as_mut_block();
                self.border_box.size.inline =
                    max(block_flow.base.intrinsic_inline_sizes.minimum_inline_size,
                        block_flow.base.intrinsic_inline_sizes.preferred_inline_size);
                block_flow.base.block_container_inline_size = self.border_box.size.inline;
                block_flow.base.block_container_writing_mode = self.style.writing_mode;
            }
            SpecificFragmentInfo::InlineAbsolute(ref mut info) => {
                let block_flow = FlowRef::deref_mut(&mut info.flow_ref).as_mut_block();
                self.border_box.size.inline =
                    max(block_flow.base.intrinsic_inline_sizes.minimum_inline_size,
                        block_flow.base.intrinsic_inline_sizes.preferred_inline_size);
                block_flow.base.block_container_inline_size = self.border_box.size.inline;
                block_flow.base.block_container_writing_mode = self.style.writing_mode;
            }

            // Text
            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: Some(ref info),
                ..
            }) |
            SpecificFragmentInfo::ScannedText(box ref info) => {
                // Scanned text fragments will have already had their content inline-sizes assigned
                // by this point.
                self.border_box.size.inline = info.content_size.inline +
                    self.border_padding.inline_start_end();
            }

            // Replaced elements
            _ if self.is_replaced() => {
                let (inline_size, block_size) =
                    self.calculate_replaced_sizes(Some(container_inline_size), container_block_size);
                self.border_box.size.inline = inline_size + self.border_padding.inline_start_end();
                self.border_box.size.block = block_size + self.border_padding.block_start_end();
            }

            ref unhandled @ _ => panic!("this case should have been handled above: {:?}", unhandled),
        }
    }

    /// Assign block-size for this fragment if it is replaced content. The inline-size must have
    /// been assigned first.
    ///
    /// Ideally, this should follow CSS 2.1 § 10.6.2.
    pub fn assign_replaced_block_size_if_necessary(&mut self) {
        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: None,
                ..
            }) |
            SpecificFragmentInfo::Generic |
            SpecificFragmentInfo::GeneratedContent(_) |
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableWrapper |
            SpecificFragmentInfo::Multicol |
            SpecificFragmentInfo::MulticolColumn => return,
            SpecificFragmentInfo::TableColumn(_) => {
                panic!("Table column fragments do not have block size")
            }
            SpecificFragmentInfo::UnscannedText(_) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            }
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::InlineBlock(_) |
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) |
            SpecificFragmentInfo::InlineAbsolute(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: Some(_),
                ..
            }) |
            SpecificFragmentInfo::Svg(_) => {}
        }

        match self.specific {
            // Text
            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: Some(ref info),
                ..
            }) |
            SpecificFragmentInfo::ScannedText(box ref info) => {
                // Scanned text fragments' content block-sizes are calculated by the text run
                // scanner during flow construction.
                self.border_box.size.block = info.content_size.block +
                    self.border_padding.block_start_end();
            }

            // Inline blocks
            SpecificFragmentInfo::InlineBlock(ref mut info) => {
                // Not the primary fragment, so we do not take the noncontent size into account.
                let block_flow = FlowRef::deref_mut(&mut info.flow_ref).as_block();
                self.border_box.size.block = block_flow.base.position.size.block +
                    block_flow.fragment.margin.block_start_end()
            }
            SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut info) => {
                // Not the primary fragment, so we do not take the noncontent size into account.
                let block_flow = FlowRef::deref_mut(&mut info.flow_ref).as_block();
                self.border_box.size.block = block_flow.base.position.size.block;
            }
            SpecificFragmentInfo::InlineAbsolute(ref mut info) => {
                // Not the primary fragment, so we do not take the noncontent size into account.
                let block_flow = FlowRef::deref_mut(&mut info.flow_ref).as_block();
                self.border_box.size.block = block_flow.base.position.size.block +
                    block_flow.fragment.margin.block_start_end()
            }

            // Replaced elements
            _ if self.is_replaced() => {},

            ref unhandled @ _ => panic!("should have been handled above: {:?}", unhandled),
        }
    }

    /// Returns true if this fragment is replaced content.
    pub fn is_replaced(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::Svg(_) => true,
            _ => false
        }
    }

    /// Returns true if this fragment is replaced content or an inline-block or false otherwise.
    pub fn is_replaced_or_inline_block(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) |
            SpecificFragmentInfo::InlineBlock(_) => true,
            _ => self.is_replaced(),
        }
    }

    /// Calculates block-size above baseline, depth below baseline, and ascent for this fragment
    /// when used in an inline formatting context. See CSS 2.1 § 10.8.1.
    ///
    /// This does not take `vertical-align` into account. For that, use `aligned_inline_metrics()`.
    fn content_inline_metrics(&self, layout_context: &LayoutContext) -> InlineMetrics {
        // CSS 2.1 § 10.8: "The height of each inline-level box in the line box is
        // calculated. For replaced elements, inline-block elements, and inline-table
        // elements, this is the height of their margin box."
        //
        // FIXME(pcwalton): We have to handle `Generic` and `GeneratedContent` here to avoid
        // crashing in a couple of `css21_dev/html4/content-` WPTs, but I don't see how those two
        // fragment types should end up inside inlines. (In the case of `GeneratedContent`, those
        // fragment types should have been resolved by now…)
        let inline_metrics = match self.specific {
            SpecificFragmentInfo::Canvas(_) | SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Image(_) | SpecificFragmentInfo::Svg(_) |
            SpecificFragmentInfo::Generic | SpecificFragmentInfo::GeneratedContent(_) => {
                let ascent = self.border_box.size.block + self.margin.block_end;
                InlineMetrics {
                    space_above_baseline: ascent + self.margin.block_start,
                    space_below_baseline: Au(0),
                    ascent: ascent,
                }
            }
            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: Some(ref info),
                ..
            }) |
            SpecificFragmentInfo::ScannedText(box ref info) => {
                // Fragments with no glyphs don't contribute any inline metrics.
                // TODO: Filter out these fragments during flow construction?
                if info.insertion_point.is_none() && info.content_size.inline == Au(0) {
                    return InlineMetrics::new(Au(0), Au(0), Au(0));
                }
                // See CSS 2.1 § 10.8.1.
                let font_metrics = with_thread_local_font_context(layout_context, |font_context| {
                    text::font_metrics_for_style(font_context, self.style.clone_font())
                });
                let line_height = text::line_height_from_style(&*self.style, &font_metrics);
                InlineMetrics::from_font_metrics(&info.run.font_metrics, line_height)
            }
            SpecificFragmentInfo::InlineBlock(ref info) => {
                inline_metrics_of_block(&info.flow_ref, &*self.style)
            }
            SpecificFragmentInfo::InlineAbsoluteHypothetical(ref info) => {
                inline_metrics_of_block(&info.flow_ref, &*self.style)
            }
            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: None,
                ..
            }) |
            SpecificFragmentInfo::InlineAbsolute(_) => {
                InlineMetrics::new(Au(0), Au(0), Au(0))
            }
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableColumn(_) |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableWrapper |
            SpecificFragmentInfo::Multicol |
            SpecificFragmentInfo::MulticolColumn |
            SpecificFragmentInfo::UnscannedText(_) => {
                unreachable!("Shouldn't see fragments of this type here!")
            }
        };
        return inline_metrics;

        fn inline_metrics_of_block(flow: &FlowRef, style: &ServoComputedValues) -> InlineMetrics {
            // CSS 2.1 § 10.8: "The height of each inline-level box in the line box is calculated.
            // For replaced elements, inline-block elements, and inline-table elements, this is the
            // height of their margin box."
            //
            // CSS 2.1 § 10.8.1: "The baseline of an 'inline-block' is the baseline of its last
            // line box in the normal flow, unless it has either no in-flow line boxes or if its
            // 'overflow' property has a computed value other than 'visible', in which case the
            // baseline is the bottom margin edge."
            //
            // NB: We must use `block_flow.fragment.border_box.size.block` here instead of
            // `block_flow.base.position.size.block` because sometimes the latter is late-computed
            // and isn't up to date at this point.
            let block_flow = flow.as_block();
            let start_margin = block_flow.fragment.margin.block_start;
            let end_margin = block_flow.fragment.margin.block_end;
            if style.get_box().overflow_y == overflow_x::T::visible {
                if let Some(baseline_offset) = flow.baseline_offset_of_last_line_box_in_flow() {
                    let ascent = baseline_offset + start_margin;
                    let space_below_baseline = block_flow.fragment.border_box.size.block -
                        baseline_offset + end_margin;
                    return InlineMetrics::new(ascent, space_below_baseline, baseline_offset)
                }
            }
            let ascent = block_flow.fragment.border_box.size.block + end_margin;
            let space_above_baseline = start_margin + ascent;
            InlineMetrics::new(space_above_baseline, Au(0), ascent)
        }
    }

    /// Calculates the offset from the baseline that applies to this fragment due to
    /// `vertical-align`. Positive values represent downward displacement.
    ///
    /// If `actual_line_metrics` is supplied, then these metrics are used to determine the
    /// displacement of the fragment when `top` or `bottom` `vertical-align` values are
    /// encountered. If this is not supplied, then `top` and `bottom` values are ignored.
    fn vertical_alignment_offset(&self,
                                 layout_context: &LayoutContext,
                                 content_inline_metrics: &InlineMetrics,
                                 minimum_line_metrics: &LineMetrics,
                                 actual_line_metrics: Option<&LineMetrics>)
                                 -> Au {
        let mut offset = Au(0);
        for style in self.inline_styles() {
            // If any of the inline styles say `top` or `bottom`, adjust the vertical align
            // appropriately.
            //
            // FIXME(#5624, pcwalton): This passes our current reftests but isn't the right thing
            // to do.
            match style.get_box().vertical_align {
                vertical_align::T::baseline => {}
                vertical_align::T::middle => {
                    let font_metrics = with_thread_local_font_context(layout_context, |font_context| {
                        text::font_metrics_for_style(font_context, self.style.clone_font())
                    });
                    offset += (content_inline_metrics.ascent -
                               content_inline_metrics.space_below_baseline -
                               font_metrics.x_height).scale_by(0.5)
                }
                vertical_align::T::sub => {
                    offset += minimum_line_metrics.space_needed()
                                                  .scale_by(FONT_SUBSCRIPT_OFFSET_RATIO)
                }
                vertical_align::T::super_ => {
                    offset -= minimum_line_metrics.space_needed()
                                                  .scale_by(FONT_SUPERSCRIPT_OFFSET_RATIO)
                }
                vertical_align::T::text_top => {
                    offset = self.content_inline_metrics(layout_context).ascent -
                        minimum_line_metrics.space_above_baseline
                }
                vertical_align::T::text_bottom => {
                    offset = minimum_line_metrics.space_below_baseline -
                        self.content_inline_metrics(layout_context).space_below_baseline
                }
                vertical_align::T::top => {
                    if let Some(actual_line_metrics) = actual_line_metrics {
                        offset = content_inline_metrics.ascent -
                            actual_line_metrics.space_above_baseline
                    }
                }
                vertical_align::T::bottom => {
                    if let Some(actual_line_metrics) = actual_line_metrics {
                        offset = actual_line_metrics.space_below_baseline -
                            content_inline_metrics.space_below_baseline
                    }
                }
                vertical_align::T::LengthOrPercentage(LengthOrPercentage::Length(length)) => {
                    offset -= length
                }
                vertical_align::T::LengthOrPercentage(LengthOrPercentage::Percentage(
                        percentage)) => {
                    offset -= minimum_line_metrics.space_needed().scale_by(percentage)
                }
                vertical_align::T::LengthOrPercentage(LengthOrPercentage::Calc(formula)) => {
                    offset -= minimum_line_metrics.space_needed().scale_by(formula.percentage()) +
                        formula.length()
                }
            }
        }
        offset
    }

    /// Calculates block-size above baseline, depth below baseline, and ascent for this fragment
    /// when used in an inline formatting context, taking `vertical-align` (other than `top` or
    /// `bottom`) into account. See CSS 2.1 § 10.8.1.
    ///
    /// If `actual_line_metrics` is supplied, then these metrics are used to determine the
    /// displacement of the fragment when `top` or `bottom` `vertical-align` values are
    /// encountered. If this is not supplied, then `top` and `bottom` values are ignored.
    pub fn aligned_inline_metrics(&self,
                                  layout_context: &LayoutContext,
                                  minimum_line_metrics: &LineMetrics,
                                  actual_line_metrics: Option<&LineMetrics>)
                                  -> InlineMetrics {
        let content_inline_metrics = self.content_inline_metrics(layout_context);
        let vertical_alignment_offset = self.vertical_alignment_offset(layout_context,
                                                                       &content_inline_metrics,
                                                                       minimum_line_metrics,
                                                                       actual_line_metrics);
        let mut space_above_baseline = match actual_line_metrics {
            None => content_inline_metrics.space_above_baseline,
            Some(actual_line_metrics) => actual_line_metrics.space_above_baseline,
        };
        space_above_baseline = space_above_baseline - vertical_alignment_offset;
        let space_below_baseline = content_inline_metrics.space_below_baseline +
            vertical_alignment_offset;
        let ascent = content_inline_metrics.ascent - vertical_alignment_offset;
        InlineMetrics::new(space_above_baseline, space_below_baseline, ascent)
    }

    /// Returns true if this fragment is a hypothetical box. See CSS 2.1 § 10.3.7.
    pub fn is_hypothetical(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) => true,
            _ => false,
        }
    }

    /// Returns true if this fragment can merge with another immediately-following fragment or
    /// false otherwise.
    pub fn can_merge_with_fragment(&self, other: &Fragment) -> bool {
        match (&self.specific, &other.specific) {
            (&SpecificFragmentInfo::UnscannedText(ref first_unscanned_text),
             &SpecificFragmentInfo::UnscannedText(_)) => {
                // FIXME: Should probably use a whitelist of styles that can safely differ (#3165)
                if self.style().get_font() != other.style().get_font() ||
                        self.text_decoration_line() != other.text_decoration_line() ||
                        self.white_space() != other.white_space() ||
                        self.color() != other.color() {
                    return false
                }

                if first_unscanned_text.text.ends_with('\n') {
                    return false
                }

                // If this node has any styles that have border/padding/margins on the following
                // side, then we can't merge with the next fragment.
                if let Some(ref inline_context) = self.inline_context {
                    for inline_context_node in inline_context.nodes.iter() {
                        if !inline_context_node.flags.contains(LAST_FRAGMENT_OF_ELEMENT) {
                            continue
                        }
                        if inline_context_node.style.logical_margin().inline_end !=
                                LengthOrPercentageOrAuto::Length(Au(0)) {
                            return false
                        }
                        if inline_context_node.style.logical_padding().inline_end !=
                                LengthOrPercentage::Length(Au(0)) {
                            return false
                        }
                        if inline_context_node.style.logical_border_width().inline_end != Au(0) {
                            return false
                        }
                    }
                }

                // If the next fragment has any styles that have border/padding/margins on the
                // preceding side, then it can't merge with us.
                if let Some(ref inline_context) = other.inline_context {
                    for inline_context_node in inline_context.nodes.iter() {
                        if !inline_context_node.flags.contains(FIRST_FRAGMENT_OF_ELEMENT) {
                            continue
                        }
                        if inline_context_node.style.logical_margin().inline_start !=
                                LengthOrPercentageOrAuto::Length(Au(0)) {
                            return false
                        }
                        if inline_context_node.style.logical_padding().inline_start !=
                                LengthOrPercentage::Length(Au(0)) {
                            return false
                        }
                        if inline_context_node.style.logical_border_width().inline_start != Au(0) {
                            return false
                        }
                    }
                }

                true
            }
            _ => false,
        }
    }

    /// Returns true if and only if this is the *primary fragment* for the fragment's style object
    /// (conceptually, though style sharing makes this not really true, of course). The primary
    /// fragment is the one that draws backgrounds, borders, etc., and takes borders, padding and
    /// margins into account. Every style object has at most one primary fragment.
    ///
    /// At present, all fragments are primary fragments except for inline-block and table wrapper
    /// fragments. Inline-block fragments are not primary fragments because the corresponding block
    /// flow is the primary fragment, while table wrapper fragments are not primary fragments
    /// because the corresponding table flow is the primary fragment.
    pub fn is_primary_fragment(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::InlineBlock(_) |
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) |
            SpecificFragmentInfo::InlineAbsolute(_) |
            SpecificFragmentInfo::MulticolColumn |
            SpecificFragmentInfo::TableWrapper => false,
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Generic |
            SpecificFragmentInfo::GeneratedContent(_) |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::Svg(_) |
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableColumn(_) |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::Multicol |
            SpecificFragmentInfo::UnscannedText(_) => true,
        }
    }

    /// Determines the inline sizes of inline-block fragments. These cannot be fully computed until
    /// inline size assignment has run for the child flow: thus it is computed "late", during
    /// block size assignment.
    pub fn update_late_computed_replaced_inline_size_if_necessary(&mut self) {
        if let SpecificFragmentInfo::InlineBlock(ref mut inline_block_info) = self.specific {
            let block_flow = FlowRef::deref_mut(&mut inline_block_info.flow_ref).as_block();
            self.border_box.size.inline = block_flow.fragment.margin_box_inline_size();
        }
    }

    pub fn update_late_computed_inline_position_if_necessary(&mut self) {
        if let SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut info) = self.specific {
            let position = self.border_box.start.i;
            FlowRef::deref_mut(&mut info.flow_ref)
                .update_late_computed_inline_position_if_necessary(position)
        }
    }

    pub fn update_late_computed_block_position_if_necessary(&mut self) {
        if let SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut info) = self.specific {
            let position = self.border_box.start.b;
            FlowRef::deref_mut(&mut info.flow_ref)
                .update_late_computed_block_position_if_necessary(position)
        }
    }

    pub fn repair_style(&mut self, new_style: &StyleArc<ServoComputedValues>) {
        self.style = (*new_style).clone()
    }

    /// Given the stacking-context-relative position of the containing flow, returns the border box
    /// of this fragment relative to the parent stacking context. This takes `position: relative`
    /// into account.
    ///
    /// If `coordinate_system` is `Parent`, this returns the border box in the parent stacking
    /// context's coordinate system. Otherwise, if `coordinate_system` is `Own` and this fragment
    /// establishes a stacking context itself, this returns a border box anchored at (0, 0). (If
    /// this fragment does not establish a stacking context, then it always belongs to its parent
    /// stacking context and thus `coordinate_system` is ignored.)
    ///
    /// This is the method you should use for display list construction as well as
    /// `getBoundingClientRect()` and so forth.
    pub fn stacking_relative_border_box(&self,
                                        stacking_relative_flow_origin: &Point2D<Au>,
                                        relative_containing_block_size: &LogicalSize<Au>,
                                        relative_containing_block_mode: WritingMode,
                                        coordinate_system: CoordinateSystem)
                                        -> Rect<Au> {
        let container_size =
            relative_containing_block_size.to_physical(relative_containing_block_mode);
        let border_box = self.border_box.to_physical(self.style.writing_mode, container_size);
        if coordinate_system == CoordinateSystem::Own && self.establishes_stacking_context() {
            return Rect::new(Point2D::zero(), border_box.size)
        }

        // FIXME(pcwalton): This can double-count relative position sometimes for inlines (e.g.
        // `<div style="position:relative">x</div>`, because the `position:relative` trickles down
        // to the inline flow. Possibly we should extend the notion of "primary fragment" to fix
        // this.
        let relative_position = self.relative_position(relative_containing_block_size);
        border_box.translate_by_size(&relative_position.to_physical(self.style.writing_mode))
                  .translate(stacking_relative_flow_origin)
    }

    /// Given the stacking-context-relative border box, returns the stacking-context-relative
    /// content box.
    pub fn stacking_relative_content_box(&self, stacking_relative_border_box: &Rect<Au>)
                                         -> Rect<Au> {
        let border_padding = self.border_padding.to_physical(self.style.writing_mode);
        Rect::new(Point2D::new(stacking_relative_border_box.origin.x + border_padding.left,
                               stacking_relative_border_box.origin.y + border_padding.top),
                  Size2D::new(stacking_relative_border_box.size.width - border_padding.horizontal(),
                              stacking_relative_border_box.size.height - border_padding.vertical()))
    }

    /// Returns true if this fragment establishes a new stacking context and false otherwise.
    pub fn establishes_stacking_context(&self) -> bool {
        // Text fragments shouldn't create stacking contexts.
        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::UnscannedText(_) => return false,
            _ => {}
        }

        if self.style().get_effects().opacity != 1.0 {
            return true
        }
        if !self.style().get_effects().filter.is_empty() {
            return true
        }
        if self.style().get_effects().mix_blend_mode != mix_blend_mode::T::normal {
            return true
        }
        if self.style().get_box().transform.0.is_some() {
            return true
        }

        // TODO(mrobinson): Determine if this is necessary, since blocks with
        // transformations already create stacking contexts.
        if let Either::First(ref _length) = self.style().get_box().perspective {
            return true
        }

        // Fixed position blocks always create stacking contexts.
        if self.style.get_box().position == position::T::fixed {
            return true
        }

        match self.style().get_used_transform_style() {
            transform_style::T::flat | transform_style::T::preserve_3d => {
                return true
            }
            transform_style::T::auto => {}
        }

        match (self.style().get_box().position,
               self.style().get_position().z_index,
               self.style().get_box().overflow_x,
               self.style().get_box().overflow_y) {
            (position::T::absolute,
             Either::Second(Auto),
             overflow_x::T::visible,
             overflow_x::T::visible) |
            (position::T::fixed,
             Either::Second(Auto),
             overflow_x::T::visible,
             overflow_x::T::visible) |
            (position::T::relative,
             Either::Second(Auto),
             overflow_x::T::visible,
             overflow_x::T::visible) => false,
            (position::T::absolute, _, _, _) |
            (position::T::fixed, _, _, _) |
            (position::T::relative, _, _, _) => true,
            (position::T::static_, _, _, _) => false
        }
    }

    // Get the effective z-index of this fragment. Z-indices only apply to positioned element
    // per CSS 2 9.9.1 (http://www.w3.org/TR/CSS2/visuren.html#z-index), so this value may differ
    // from the value specified in the style.
    pub fn effective_z_index(&self) -> i32 {
        match self.style().get_box().position {
            position::T::static_ => {},
            _ => return self.style().get_position().z_index.integer_or(0),
        }

        if self.style().get_box().transform.0.is_some() {
            return self.style().get_position().z_index.integer_or(0);
        }

        match self.style().get_box().display {
            display::T::flex => self.style().get_position().z_index.integer_or(0),
            _ => 0,
        }
    }

    /// Computes the overflow rect of this fragment relative to the start of the flow.
    pub fn compute_overflow(&self,
                            flow_size: &Size2D<Au>,
                            relative_containing_block_size: &LogicalSize<Au>)
                            -> Overflow {
        let mut border_box = self.border_box.to_physical(self.style.writing_mode, *flow_size);

        // Relative position can cause us to draw outside our border box.
        //
        // FIXME(pcwalton): I'm not a fan of the way this makes us crawl though so many styles all
        // the time. Can't we handle relative positioning by just adjusting `border_box`?
        let relative_position = self.relative_position(relative_containing_block_size);
        border_box =
            border_box.translate_by_size(&relative_position.to_physical(self.style.writing_mode));
        let mut overflow = Overflow::from_rect(&border_box);

        // Box shadows cause us to draw outside our border box.
        for box_shadow in &self.style().get_effects().box_shadow.0 {
            let offset = Point2D::new(box_shadow.offset_x, box_shadow.offset_y);
            let inflation = box_shadow.spread_radius + box_shadow.blur_radius *
                BLUR_INFLATION_FACTOR;
            overflow.paint = overflow.paint.union(&border_box.translate(&offset)
                                                             .inflate(inflation, inflation))
        }

        // Outlines cause us to draw outside our border box.
        let outline_width = self.style.get_outline().outline_width;
        if outline_width != Au(0) {
            overflow.paint = overflow.paint.union(&border_box.inflate(outline_width,
                                                                      outline_width))
        }

        // Include the overflow of the block flow, if any.
        match self.specific {
            SpecificFragmentInfo::InlineBlock(ref info) => {
                let block_flow = info.flow_ref.as_block();
                overflow.union(&flow::base(block_flow).overflow);
            }
            SpecificFragmentInfo::InlineAbsolute(ref info) => {
                let block_flow = info.flow_ref.as_block();
                overflow.union(&flow::base(block_flow).overflow);
            }
            _ => (),
        }

        // FIXME(pcwalton): Sometimes excessively fancy glyphs can make us draw outside our border
        // box too.
        overflow
    }

    pub fn requires_line_break_afterward_if_wrapping_on_newlines(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: Some(ref scanned_text),
                ..
            }) |
            SpecificFragmentInfo::ScannedText(box ref scanned_text) => {
                scanned_text.requires_line_break_afterward_if_wrapping_on_newlines()
            }
            _ => false,
        }
    }

    pub fn strip_leading_whitespace_if_necessary(&mut self) -> WhitespaceStrippingResult {
        if self.white_space().preserve_spaces() {
            return WhitespaceStrippingResult::RetainFragment
        }

        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: Some(ref mut scanned_text_fragment_info),
                ..
            }) |
            SpecificFragmentInfo::ScannedText(box ref mut scanned_text_fragment_info) => {
                let leading_whitespace_byte_count = scanned_text_fragment_info.text()
                    .find(|c| !char_is_whitespace(c))
                    .unwrap_or(scanned_text_fragment_info.text().len());

                let whitespace_len = ByteIndex(leading_whitespace_byte_count as isize);
                let whitespace_range = Range::new(scanned_text_fragment_info.range.begin(),
                                                  whitespace_len);
                let text_bounds =
                    scanned_text_fragment_info.run.metrics_for_range(&whitespace_range).bounding_box;
                self.border_box.size.inline = self.border_box.size.inline - text_bounds.size.width;
                scanned_text_fragment_info.content_size.inline =
                    scanned_text_fragment_info.content_size.inline - text_bounds.size.width;

                scanned_text_fragment_info.range.adjust_by(whitespace_len, -whitespace_len);

                WhitespaceStrippingResult::RetainFragment
            }
            SpecificFragmentInfo::UnscannedText(ref mut unscanned_text_fragment_info) => {
                let mut new_text_string = String::new();
                let mut modified = false;
                for (i, character) in unscanned_text_fragment_info.text.char_indices() {
                    if gfx::text::util::is_bidi_control(character) {
                        new_text_string.push(character);
                        continue
                    }
                    if char_is_whitespace(character) {
                        modified = true;
                        continue
                    }
                    // Finished processing leading control chars and whitespace.
                    if modified {
                        new_text_string.push_str(&unscanned_text_fragment_info.text[i..]);
                    }
                    break
                }
                if modified {
                    unscanned_text_fragment_info.text = new_text_string.into_boxed_str();
                }

                WhitespaceStrippingResult::from_unscanned_text_fragment_info(
                    &unscanned_text_fragment_info)
            }
            _ => WhitespaceStrippingResult::RetainFragment,
        }
    }

    /// Returns true if the entire fragment was stripped.
    pub fn strip_trailing_whitespace_if_necessary(&mut self) -> WhitespaceStrippingResult {
        if self.white_space().preserve_spaces() {
            return WhitespaceStrippingResult::RetainFragment
        }

        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(box TruncatedFragmentInfo {
                text_info: Some(ref mut scanned_text_fragment_info),
                ..
            }) |
            SpecificFragmentInfo::ScannedText(box ref mut scanned_text_fragment_info) => {
                let mut trailing_whitespace_start_byte = 0;
                for (i, c) in scanned_text_fragment_info.text().char_indices().rev() {
                    if !char_is_whitespace(c) {
                        trailing_whitespace_start_byte = i + c.len_utf8();
                        break;
                    }
                }
                let whitespace_start = ByteIndex(trailing_whitespace_start_byte as isize);
                let whitespace_len = scanned_text_fragment_info.range.length() - whitespace_start;
                let mut whitespace_range = Range::new(whitespace_start, whitespace_len);
                whitespace_range.shift_by(scanned_text_fragment_info.range.begin());

                let text_bounds = scanned_text_fragment_info.run
                                                        .metrics_for_range(&whitespace_range)
                                                        .bounding_box;
                self.border_box.size.inline -= text_bounds.size.width;
                scanned_text_fragment_info.content_size.inline -= text_bounds.size.width;

                scanned_text_fragment_info.range.extend_by(-whitespace_len);
                WhitespaceStrippingResult::RetainFragment
            }
            SpecificFragmentInfo::UnscannedText(ref mut unscanned_text_fragment_info) => {
                let mut trailing_bidi_control_characters_to_retain = Vec::new();
                let (mut modified, mut last_character_index) = (true, 0);
                for (i, character) in unscanned_text_fragment_info.text.char_indices().rev() {
                    if gfx::text::util::is_bidi_control(character) {
                        trailing_bidi_control_characters_to_retain.push(character);
                        continue
                    }
                    if char_is_whitespace(character) {
                        modified = true;
                        continue
                    }
                    last_character_index = i + character.len_utf8();
                    break
                }
                if modified {
                    let mut text = unscanned_text_fragment_info.text.to_string();
                    text.truncate(last_character_index);
                    for character in trailing_bidi_control_characters_to_retain.iter().rev() {
                        text.push(*character);
                    }
                    unscanned_text_fragment_info.text = text.into_boxed_str();
                }

                WhitespaceStrippingResult::from_unscanned_text_fragment_info(
                    &unscanned_text_fragment_info)
            }
            _ => WhitespaceStrippingResult::RetainFragment,
        }
    }

    pub fn inline_styles(&self) -> InlineStyleIterator {
        InlineStyleIterator::new(self)
    }

    /// Returns the inline-size of this fragment's margin box.
    pub fn margin_box_inline_size(&self) -> Au {
        self.border_box.size.inline + self.margin.inline_start_end()
    }

    /// Returns true if this node *or any of the nodes within its inline fragment context* have
    /// non-`static` `position`.
    pub fn is_positioned(&self) -> bool {
        if self.style.get_box().position != position::T::static_ {
            return true
        }
        if let Some(ref inline_context) = self.inline_context {
            for node in inline_context.nodes.iter() {
                if node.style.get_box().position != position::T::static_ {
                    return true
                }
            }
        }
        false
    }

    /// Returns true if this node is absolutely positioned.
    pub fn is_absolutely_positioned(&self) -> bool {
        self.style.get_box().position == position::T::absolute
    }

    pub fn is_inline_absolute(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::InlineAbsolute(..) => true,
            _ => false,
        }
    }

    pub fn meld_with_next_inline_fragment(&mut self, next_fragment: &Fragment) {
        if let Some(ref mut inline_context_of_this_fragment) = self.inline_context {
            if let Some(ref inline_context_of_next_fragment) = next_fragment.inline_context {
                for (inline_context_node_from_this_fragment,
                     inline_context_node_from_next_fragment)
                    in inline_context_of_this_fragment.nodes.iter_mut().rev()
                        .zip(inline_context_of_next_fragment.nodes.iter().rev())
                {
                    if !inline_context_node_from_next_fragment.flags.contains(
                            LAST_FRAGMENT_OF_ELEMENT) {
                        continue
                    }
                    if inline_context_node_from_next_fragment.address !=
                            inline_context_node_from_this_fragment.address {
                        continue
                    }
                    inline_context_node_from_this_fragment.flags.insert(LAST_FRAGMENT_OF_ELEMENT);
                }
            }
        }
    }

    pub fn meld_with_prev_inline_fragment(&mut self, prev_fragment: &Fragment) {
        if let Some(ref mut inline_context_of_this_fragment) = self.inline_context {
            if let Some(ref inline_context_of_prev_fragment) = prev_fragment.inline_context {
                for (inline_context_node_from_prev_fragment,
                     inline_context_node_from_this_fragment)
                    in inline_context_of_prev_fragment.nodes.iter().rev().zip(
                            inline_context_of_this_fragment.nodes.iter_mut().rev())
                {
                    if !inline_context_node_from_prev_fragment.flags.contains(
                            FIRST_FRAGMENT_OF_ELEMENT) {
                        continue
                    }
                    if inline_context_node_from_prev_fragment.address !=
                            inline_context_node_from_this_fragment.address {
                        continue
                    }
                    inline_context_node_from_this_fragment.flags.insert(
                        FIRST_FRAGMENT_OF_ELEMENT);
                }
            }
        }
    }

    /// Returns true if any of the inline styles associated with this fragment have
    /// `vertical-align` set to `top` or `bottom`.
    pub fn is_vertically_aligned_to_top_or_bottom(&self) -> bool {
        match self.style.get_box().vertical_align {
            vertical_align::T::top | vertical_align::T::bottom => return true,
            _ => {}
        }
        if let Some(ref inline_context) = self.inline_context {
            for node in &inline_context.nodes {
                match node.style.get_box().vertical_align {
                    vertical_align::T::top | vertical_align::T::bottom => return true,
                    _ => {}
                }
            }
        }
        false
    }

    pub fn is_text_or_replaced(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::Generic |
            SpecificFragmentInfo::InlineAbsolute(_) |
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) |
            SpecificFragmentInfo::InlineBlock(_) |
            SpecificFragmentInfo::Multicol |
            SpecificFragmentInfo::MulticolColumn |
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableColumn(_) |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableWrapper => false,
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::GeneratedContent(_) |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::Svg(_) |
            SpecificFragmentInfo::UnscannedText(_) => true
        }
    }

    /// Returns the 4D matrix representing this fragment's transform.
    pub fn transform_matrix(&self, stacking_relative_border_box: &Rect<Au>) -> Option<Matrix4D<f32>> {
        let operations = match self.style.get_box().transform.0 {
            None => return None,
            Some(ref operations) => operations,
        };

        let mut transform = Matrix4D::identity();
        let transform_origin = &self.style.get_box().transform_origin;
        let transform_origin_x = model::specified(transform_origin.horizontal,
                                                  stacking_relative_border_box.size
                                                                              .width).to_f32_px();
        let transform_origin_y = model::specified(transform_origin.vertical,
                                                  stacking_relative_border_box.size
                                                                              .height).to_f32_px();
        let transform_origin_z = transform_origin.depth.to_f32_px();

        let pre_transform = Matrix4D::create_translation(transform_origin_x,
                                                         transform_origin_y,
                                                         transform_origin_z);
        let post_transform = Matrix4D::create_translation(-transform_origin_x,
                                                          -transform_origin_y,
                                                          -transform_origin_z);

        for operation in operations {
            let matrix = match *operation {
                transform::ComputedOperation::Rotate(ax, ay, az, theta) => {
                    let theta = 2.0f32 * f32::consts::PI - theta.radians();
                    Matrix4D::create_rotation(ax, ay, az, Radians::new(theta))
                }
                transform::ComputedOperation::Perspective(d) => {
                    create_perspective_matrix(d)
                }
                transform::ComputedOperation::Scale(sx, sy, sz) => {
                    Matrix4D::create_scale(sx, sy, sz)
                }
                transform::ComputedOperation::Translate(tx, ty, tz) => {
                    let tx =
                        model::specified(tx, stacking_relative_border_box.size.width).to_f32_px();
                    let ty =
                        model::specified(ty, stacking_relative_border_box.size.height).to_f32_px();
                    let tz = tz.to_f32_px();
                    Matrix4D::create_translation(tx, ty, tz)
                }
                transform::ComputedOperation::Matrix(m) => {
                    m.to_gfx_matrix()
                }
                transform::ComputedOperation::MatrixWithPercents(_) => {
                    // `-moz-transform` is not implemented in Servo yet.
                    unreachable!()
                }
                transform::ComputedOperation::Skew(theta_x, theta_y) => {
                    Matrix4D::create_skew(Radians::new(theta_x.radians()),
                                          Radians::new(theta_y.radians()))
                }
            };

            transform = transform.pre_mul(&matrix);
        }

        Some(pre_transform.pre_mul(&transform).pre_mul(&post_transform))
    }

    /// Returns the 4D matrix representing this fragment's perspective.
    pub fn perspective_matrix(&self, stacking_relative_border_box: &Rect<Au>) -> Option<Matrix4D<f32>> {
        match self.style().get_box().perspective {
            Either::First(length) => {
                let perspective_origin = self.style().get_box().perspective_origin;
                let perspective_origin =
                    Point2D::new(model::specified(perspective_origin.horizontal,
                                                  stacking_relative_border_box.size.width).to_f32_px(),
                                 model::specified(perspective_origin.vertical,
                                                  stacking_relative_border_box.size.height).to_f32_px());

                let pre_transform = Matrix4D::create_translation(perspective_origin.x,
                                                                 perspective_origin.y,
                                                                 0.0);
                let post_transform = Matrix4D::create_translation(-perspective_origin.x,
                                                                  -perspective_origin.y,
                                                                  0.0);

                let perspective_matrix = create_perspective_matrix(length);

                Some(pre_transform.pre_mul(&perspective_matrix).pre_mul(&post_transform))
            }
            Either::Second(values::None_) => {
                None
            }
        }
    }
}

impl fmt::Debug for Fragment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let border_padding_string = if !self.border_padding.is_zero() {
            format!(" border_padding={:?}", self.border_padding)
        } else {
            "".to_owned()
        };

        let margin_string = if !self.margin.is_zero() {
            format!(" margin={:?}", self.margin)
        } else {
            "".to_owned()
        };

        let damage_string = if self.restyle_damage != RestyleDamage::empty() {
            format!(" damage={:?}", self.restyle_damage)
        } else {
            "".to_owned()
        };

        write!(f, "{}({}) [{:?}] border_box={:?}{}{}{}",
            self.specific.get_type(),
            self.debug_id,
            self.specific,
            self.border_box,
            border_padding_string,
            margin_string,
            damage_string)
    }
}

bitflags! {
    flags QuantitiesIncludedInIntrinsicInlineSizes: u8 {
        const INTRINSIC_INLINE_SIZE_INCLUDES_MARGINS = 0x01,
        const INTRINSIC_INLINE_SIZE_INCLUDES_PADDING = 0x02,
        const INTRINSIC_INLINE_SIZE_INCLUDES_BORDER = 0x04,
        const INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED = 0x08,
    }
}

bitflags! {
    // Various flags we can use when splitting fragments. See
    // `calculate_split_position_using_breaking_strategy()`.
    flags SplitOptions: u8 {
        #[doc = "True if this is the first fragment on the line."]
        const STARTS_LINE = 0x01,
        #[doc = "True if we should attempt to split at character boundaries if this split fails. \
                 This is used to implement `overflow-wrap: break-word`."]
        const RETRY_AT_CHARACTER_BOUNDARIES = 0x02,
    }
}

/// A top-down fragment border box iteration handler.
pub trait FragmentBorderBoxIterator {
    /// The operation to perform.
    fn process(&mut self, fragment: &Fragment, level: i32, overflow: &Rect<Au>);

    /// Returns true if this fragment must be processed in-order. If this returns false,
    /// we skip the operation for this fragment, but continue processing siblings.
    fn should_process(&mut self, fragment: &Fragment) -> bool;
}

/// The coordinate system used in `stacking_relative_border_box()`. See the documentation of that
/// method for details.
#[derive(Clone, PartialEq, Debug)]
pub enum CoordinateSystem {
    /// The border box returned is relative to the fragment's parent stacking context.
    Parent,
    /// The border box returned is relative to the fragment's own stacking context, if applicable.
    Own,
}

pub struct InlineStyleIterator<'a> {
    fragment: &'a Fragment,
    inline_style_index: usize,
    primary_style_yielded: bool,
}

impl<'a> Iterator for InlineStyleIterator<'a> {
    type Item = &'a ServoComputedValues;

    fn next(&mut self) -> Option<&'a ServoComputedValues> {
        if !self.primary_style_yielded {
            self.primary_style_yielded = true;
            return Some(&*self.fragment.style)
        }
        let inline_context = match self.fragment.inline_context {
            None => return None,
            Some(ref inline_context) => inline_context,
        };
        let inline_style_index = self.inline_style_index;
        if inline_style_index == inline_context.nodes.len() {
            return None
        }
        self.inline_style_index += 1;
        Some(&*inline_context.nodes[inline_style_index].style)
    }
}

impl<'a> InlineStyleIterator<'a> {
    fn new(fragment: &Fragment) -> InlineStyleIterator {
        InlineStyleIterator {
            fragment: fragment,
            inline_style_index: 0,
            primary_style_yielded: false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WhitespaceStrippingResult {
    RetainFragment,
    FragmentContainedOnlyBidiControlCharacters,
    FragmentContainedOnlyWhitespace,
}

impl WhitespaceStrippingResult {
    fn from_unscanned_text_fragment_info(info: &UnscannedTextFragmentInfo)
                                         -> WhitespaceStrippingResult {
        if info.text.is_empty() {
            WhitespaceStrippingResult::FragmentContainedOnlyWhitespace
        } else if info.text.chars().all(gfx::text::util::is_bidi_control) {
            WhitespaceStrippingResult::FragmentContainedOnlyBidiControlCharacters
        } else {
            WhitespaceStrippingResult::RetainFragment
        }
    }
}

/// The overflow area. We need two different notions of overflow: paint overflow and scrollable
/// overflow.
#[derive(Copy, Clone, Debug)]
pub struct Overflow {
    pub scroll: Rect<Au>,
    pub paint: Rect<Au>,
}

impl Overflow {
    pub fn new() -> Overflow {
        Overflow {
            scroll: Rect::zero(),
            paint: Rect::zero(),
        }
    }

    pub fn from_rect(border_box: &Rect<Au>) -> Overflow {
        Overflow {
            scroll: *border_box,
            paint: *border_box,
        }
    }

    pub fn union(&mut self, other: &Overflow) {
        self.scroll = self.scroll.union(&other.scroll);
        self.paint = self.paint.union(&other.paint);
    }

    pub fn translate(&mut self, point: &Point2D<Au>) {
        self.scroll = self.scroll.translate(point);
        self.paint = self.paint.translate(point);
    }
}

bitflags! {
    pub flags FragmentFlags: u8 {
        // TODO(stshine): find a better name since these flags can also be used for grid item.
        /// Whether this fragment represents a child in a row flex container.
        const IS_INLINE_FLEX_ITEM = 0b0000_0001,
        /// Whether this fragment represents a child in a column flex container.
        const IS_BLOCK_FLEX_ITEM = 0b0000_0010,
        /// Whether this fragment represents the generated text from a text-overflow clip.
        const IS_ELLIPSIS = 0b0000_0100,
    }
}

/// Specified distances from the margin edge of a block to its content in the inline direction.
/// These are returned by `guess_inline_content_edge_offsets()` and are used in the float placement
/// speculation logic.
#[derive(Copy, Clone, Debug)]
pub struct SpeculatedInlineContentEdgeOffsets {
    pub start: Au,
    pub end: Au,
}

#[cfg(not(debug_assertions))]
#[derive(Clone)]
struct DebugId;

#[cfg(debug_assertions)]
#[derive(Clone)]
struct DebugId(u16);

#[cfg(not(debug_assertions))]
impl DebugId {
    pub fn new() -> DebugId {
        DebugId
    }
}

#[cfg(debug_assertions)]
impl DebugId {
    pub fn new() -> DebugId {
        DebugId(layout_debug::generate_unique_debug_id())
    }
}

#[cfg(not(debug_assertions))]
impl fmt::Display for DebugId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}", &self)
    }
}

#[cfg(debug_assertions)]
impl fmt::Display for DebugId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(not(debug_assertions))]
impl Serialize for DebugId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{:p}", &self))
    }
}

#[cfg(debug_assertions)]
impl Serialize for DebugId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.0)
    }
}

// TODO(gw): The transforms spec says that perspective length must
// be positive. However, there is some confusion between the spec
// and browser implementations as to handling the case of 0 for the
// perspective value. Until the spec bug is resolved, at least ensure
// that a provided perspective value of <= 0.0 doesn't cause panics
// and behaves as it does in other browsers.
// See https://lists.w3.org/Archives/Public/www-style/2016Jan/0020.html for more details.
#[inline]
fn create_perspective_matrix(d: Au) -> Matrix4D<f32> {
    let d = d.to_f32_px();
    if d <= 0.0 {
        Matrix4D::identity()
    } else {
        Matrix4D::create_perspective(d)
    }
}
