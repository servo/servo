/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Fragment` type, which represents the leaves of the layout tree.

#![deny(unsafe_blocks)]

use css::node_style::StyledNode;
use construct::FlowConstructor;
use context::LayoutContext;
use floats::{ClearBoth, ClearLeft, ClearRight, ClearType};
use flow;
use flow::Flow;
use flow_ref::FlowRef;
use incremental::RestyleDamage;
use inline::{InlineFragmentContext, InlineMetrics};
use layout_debug;
use model::{Auto, IntrinsicISizes, IntrinsicISizesContribution, MaybeAuto, Specified, specified};
use model;
use text;
use util::OpaqueNodeMethods;
use wrapper::{TLayoutNode, ThreadSafeLayoutNode};

use geom::{Point2D, Rect, Size2D};
use gfx::display_list::OpaqueNode;
use gfx::text::glyph::CharIndex;
use gfx::text::text_run::TextRun;
use script_traits::UntrustedNodeAddress;
use serialize::{Encodable, Encoder};
use servo_msg::constellation_msg::{PipelineId, SubpageId};
use servo_net::image::holder::ImageHolder;
use servo_net::local_image_cache::LocalImageCache;
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::logical_geometry::{LogicalRect, LogicalSize, LogicalMargin};
use servo_util::range::*;
use servo_util::smallvec::SmallVec;
use servo_util::str::is_whitespace;
use std::cmp::{max, min};
use std::fmt;
use std::from_str::FromStr;
use string_cache::Atom;
use style::{ComputedValues, TElement, TNode, cascade_anonymous};
use style::computed_values::{LengthOrPercentage, LengthOrPercentageOrAuto};
use style::computed_values::{LengthOrPercentageOrNone};
use style::computed_values::{LPA_Auto, clear, position, text_align, text_decoration};
use style::computed_values::{vertical_align, white_space};
use sync::{Arc, Mutex};
use url::Url;

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
/// A `GenericFragment` is an empty fragment that contributes only borders, margins, padding, and
/// backgrounds. It is analogous to a CSS nonreplaced content box.
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
#[deriving(Clone)]
pub struct Fragment {
    /// An opaque reference to the DOM node that this `Fragment` originates from.
    pub node: OpaqueNode,

    /// The CSS style of this fragment.
    pub style: Arc<ComputedValues>,

    /// The position of this fragment relative to its owning flow.
    /// The size includes padding and border, but not margin.
    pub border_box: LogicalRect<Au>,

    /// The sum of border and padding; i.e. the distance from the edge of the border box to the
    /// content edge of the fragment.
    pub border_padding: LogicalMargin<Au>,

    /// The margin of the content box.
    pub margin: LogicalMargin<Au>,

    /// Info specific to the kind of fragment. Keep this enum small.
    pub specific: SpecificFragmentInfo,

    /// Holds the style context information for fragments
    /// that are part of an inline formatting context.
    pub inline_context: Option<InlineFragmentContext>,

    /// A debug ID that is consistent for the life of
    /// this fragment (via transform etc).
    pub debug_id: u16,

    /// How damaged this fragment is since last reflow.
    pub restyle_damage: RestyleDamage,
}

impl<E, S: Encoder<E>> Encodable<S, E> for Fragment {
    fn encode(&self, e: &mut S) -> Result<(), E> {
        e.emit_struct("fragment", 0, |e| {
            try!(e.emit_struct_field("id", 0, |e| self.debug_id().encode(e)))
            try!(e.emit_struct_field("border_box", 1, |e| self.border_box.encode(e)))
            e.emit_struct_field("margin", 2, |e| self.margin.encode(e))
        })
    }
}

/// Info specific to the kind of fragment.
///
/// Keep this enum small. As in, no more than one word. Or pcwalton will yell at you.
#[deriving(Clone)]
pub enum SpecificFragmentInfo {
    GenericFragment,
    IframeFragment(Box<IframeFragmentInfo>),
    ImageFragment(Box<ImageFragmentInfo>),

    /// A hypothetical box (see CSS 2.1 § 10.3.7) for an absolutely-positioned block that was
    /// declared with `display: inline;`.
    InlineAbsoluteHypotheticalFragment(InlineAbsoluteHypotheticalFragmentInfo),

    InlineBlockFragment(InlineBlockFragmentInfo),
    ScannedTextFragment(Box<ScannedTextFragmentInfo>),
    TableFragment,
    TableCellFragment,
    TableColumnFragment(TableColumnFragmentInfo),
    TableRowFragment,
    TableWrapperFragment,
    UnscannedTextFragment(UnscannedTextFragmentInfo),
}

impl SpecificFragmentInfo {
    fn restyle_damage(&self) -> RestyleDamage {
        let flow =
            match *self {
                IframeFragment(_)
                | ImageFragment(_)
                | ScannedTextFragment(_)
                | TableFragment
                | TableCellFragment
                | TableColumnFragment(_)
                | TableRowFragment
                | TableWrapperFragment
                | UnscannedTextFragment(_)
                | GenericFragment => return RestyleDamage::empty(),
                InlineAbsoluteHypotheticalFragment(ref info) => &info.flow_ref,
                InlineBlockFragment(ref info) => &info.flow_ref,
            };

        flow::base(flow.deref()).restyle_damage
    }

    pub fn get_type(&self) -> &'static str {
        match *self {
            GenericFragment => "GenericFragment",
            IframeFragment(_) => "IframeFragment",
            ImageFragment(_) => "ImageFragment",
            InlineAbsoluteHypotheticalFragment(_) => "InlineAbsoluteHypotheticalFragment",
            InlineBlockFragment(_) => "InlineBlockFragment",
            ScannedTextFragment(_) => "ScannedTextFragment",
            TableFragment => "TableFragment",
            TableCellFragment => "TableCellFragment",
            TableColumnFragment(_) => "TableColumnFragment",
            TableRowFragment => "TableRowFragment",
            TableWrapperFragment => "TableWrapperFragment",
            UnscannedTextFragment(_) => "UnscannedTextFragment",
        }
    }
}

/// A hypothetical box (see CSS 2.1 § 10.3.7) for an absolutely-positioned block that was declared
/// with `display: inline;`.
///
/// FIXME(pcwalton): Stop leaking this `FlowRef` to layout; that is not memory safe because layout
/// can clone it.
#[deriving(Clone)]
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
#[deriving(Clone)]
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

/// A fragment that represents a replaced content image and its accompanying borders, shadows, etc.
#[deriving(Clone)]
pub struct ImageFragmentInfo {
    /// The image held within this fragment.
    pub image: ImageHolder<UntrustedNodeAddress>,
    pub for_node: UntrustedNodeAddress,
    pub computed_inline_size: Option<Au>,
    pub computed_block_size: Option<Au>,
    pub dom_inline_size: Option<Au>,
    pub dom_block_size: Option<Au>,
    pub writing_mode_is_vertical: bool,
}

impl ImageFragmentInfo {
    /// Creates a new image fragment from the given URL and local image cache.
    ///
    /// FIXME(pcwalton): The fact that image fragments store the cache in the fragment makes little
    /// sense to me.
    pub fn new(node: &ThreadSafeLayoutNode,
               image_url: Url,
               local_image_cache: Arc<Mutex<LocalImageCache<UntrustedNodeAddress>>>)
               -> ImageFragmentInfo {
        fn convert_length(node: &ThreadSafeLayoutNode, name: &Atom) -> Option<Au> {
            let element = node.as_element();
            element.get_attr(&ns!(""), name).and_then(|string| {
                let n: Option<int> = FromStr::from_str(string);
                n
            }).and_then(|pixels| Some(Au::from_px(pixels)))
        }

        let is_vertical = node.style().writing_mode.is_vertical();
        let dom_width = convert_length(node, &atom!("width"));
        let dom_height = convert_length(node, &atom!("height"));

        let opaque_node: OpaqueNode = OpaqueNodeMethods::from_thread_safe_layout_node(node);
        let untrusted_node: UntrustedNodeAddress = opaque_node.to_untrusted_node_address();

        ImageFragmentInfo {
            image: ImageHolder::new(image_url, local_image_cache),
            for_node: untrusted_node,
            computed_inline_size: None,
            computed_block_size: None,
            dom_inline_size: if is_vertical { dom_height } else { dom_width },
            dom_block_size: if is_vertical { dom_width } else { dom_height },
            writing_mode_is_vertical: is_vertical,
        }
    }

    /// Returns the calculated inline-size of the image, accounting for the inline-size attribute.
    pub fn computed_inline_size(&self) -> Au {
        self.computed_inline_size.expect("image inline_size is not computed yet!")
    }

    /// Returns the calculated block-size of the image, accounting for the block-size attribute.
    pub fn computed_block_size(&self) -> Au {
        self.computed_block_size.expect("image block_size is not computed yet!")
    }

    /// Returns the original inline-size of the image.
    pub fn image_inline_size(&mut self) -> Au {
        let size = self.image.get_size(self.for_node).unwrap_or(Size2D::zero());
        Au::from_px(if self.writing_mode_is_vertical { size.height } else { size.width })
    }

    /// Returns the original block-size of the image.
    pub fn image_block_size(&mut self) -> Au {
        let size = self.image.get_size(self.for_node).unwrap_or(Size2D::zero());
        Au::from_px(if self.writing_mode_is_vertical { size.width } else { size.height })
    }

    // Return used value for inline-size or block-size.
    //
    // `dom_length`: inline-size or block-size as specified in the `img` tag.
    // `style_length`: inline-size as given in the CSS
    pub fn style_length(style_length: LengthOrPercentageOrAuto,
                        dom_length: Option<Au>,
                        container_inline_size: Au) -> MaybeAuto {
        match (MaybeAuto::from_style(style_length,container_inline_size),dom_length) {
            (Specified(length),_) => {
                Specified(length)
            },
            (Auto,Some(length)) => {
                Specified(length)
            },
            (Auto,None) => {
                Auto
            }
        }
    }

    /// Clamp a value obtained from style_length, based on min / max lengths.
    pub fn clamp_size(size: Au, min_size: LengthOrPercentage, max_size: LengthOrPercentageOrNone,
                        container_inline_size: Au) -> Au {
        let min_size = model::specified(min_size, container_inline_size);
        let max_size = model::specified_or_none(max_size, container_inline_size);

        Au::max(min_size, match max_size {
            None => size,
            Some(max_size) => Au::min(size, max_size),
        })
    }

    /// Tile an image
    pub fn tile_image(position: &mut Au, size: &mut Au,
                        virtual_position: Au, image_size: u32) {
        let image_size = image_size as int;
        let delta_pixels = geometry::to_px(virtual_position - *position);
        let tile_count = (delta_pixels + image_size - 1) / image_size;
        let offset = Au::from_px(image_size * tile_count);
        let new_position = virtual_position - offset;
        *size = *position - new_position + *size;
        *position = new_position;
    }
}

/// A fragment that represents an inline frame (iframe). This stores the pipeline ID so that the size
/// of this iframe can be communicated via the constellation to the iframe's own layout task.
#[deriving(Clone)]
pub struct IframeFragmentInfo {
    /// The pipeline ID of this iframe.
    pub pipeline_id: PipelineId,
    /// The subpage ID of this iframe.
    pub subpage_id: SubpageId,
}

impl IframeFragmentInfo {
    /// Creates the information specific to an iframe fragment.
    pub fn new(node: &ThreadSafeLayoutNode) -> IframeFragmentInfo {
        let (pipeline_id, subpage_id) = node.iframe_pipeline_and_subpage_ids();
        IframeFragmentInfo {
            pipeline_id: pipeline_id,
            subpage_id: subpage_id,
        }
    }
}

/// A scanned text fragment represents a single run of text with a distinct style. A `TextFragment`
/// may be split into two or more fragments across line breaks. Several `TextFragment`s may
/// correspond to a single DOM text node. Split text fragments are implemented by referring to
/// subsets of a single `TextRun` object.
#[deriving(Clone)]
pub struct ScannedTextFragmentInfo {
    /// The text run that this represents.
    pub run: Arc<Box<TextRun>>,

    /// The range within the above text run that this represents.
    pub range: Range<CharIndex>,

    /// The positions of newlines within this scanned text fragment.
    ///
    /// FIXME(#2260, pcwalton): Can't this go somewhere else, like in the text run or something?
    /// Or can we just remove it?
    pub new_line_pos: Vec<CharIndex>,

    /// The new_line_pos is eaten during line breaking. If we need to re-merge
    /// fragments, it will have to be restored.
    pub original_new_line_pos: Option<Vec<CharIndex>>,

    /// The intrinsic size of the text fragment.
    pub content_size: LogicalSize<Au>,
}

impl ScannedTextFragmentInfo {
    /// Creates the information specific to a scanned text fragment from a range and a text run.
    pub fn new(run: Arc<Box<TextRun>>,
               range: Range<CharIndex>,
               new_line_positions: Vec<CharIndex>,
               content_size: LogicalSize<Au>)
               -> ScannedTextFragmentInfo {
        ScannedTextFragmentInfo {
            run: run,
            range: range,
            new_line_pos: new_line_positions,
            original_new_line_pos: None,
            content_size: content_size,
        }
    }
}

#[deriving(Show)]
pub struct SplitInfo {
    // TODO(bjz): this should only need to be a single character index, but both values are
    // currently needed for splitting in the `inline::try_append_*` functions.
    pub range: Range<CharIndex>,
    pub inline_size: Au,
}

impl SplitInfo {
    fn new(range: Range<CharIndex>, info: &ScannedTextFragmentInfo) -> SplitInfo {
        SplitInfo {
            range: range,
            inline_size: info.run.advance_for_range(&range),
        }
    }
}

/// Data for an unscanned text fragment. Unscanned text fragments are the results of flow
/// construction that have not yet had their inline-size determined.
#[deriving(Clone)]
pub struct UnscannedTextFragmentInfo {
    /// The text inside the fragment.
    ///
    /// FIXME(pcwalton): Is there something more clever we can do here that avoids the double
    /// indirection while not penalizing all fragments?
    pub text: Box<String>,
}

impl UnscannedTextFragmentInfo {
    /// Creates a new instance of `UnscannedTextFragmentInfo` from the given DOM node.
    pub fn new(node: &ThreadSafeLayoutNode) -> UnscannedTextFragmentInfo {
        // FIXME(pcwalton): Don't copy text; atomically reference count it instead.
        UnscannedTextFragmentInfo {
            text: box node.text(),
        }
    }

    /// Creates a new instance of `UnscannedTextFragmentInfo` from the given text.
    #[inline]
    pub fn from_text(text: String) -> UnscannedTextFragmentInfo {
        UnscannedTextFragmentInfo {
            text: box text,
        }
    }
}

/// A fragment that represents a table column.
#[deriving(Clone)]
pub struct TableColumnFragmentInfo {
    /// the number of columns a <col> element should span
    pub span: int,
}

impl TableColumnFragmentInfo {
    /// Create the information specific to an table column fragment.
    pub fn new(node: &ThreadSafeLayoutNode) -> TableColumnFragmentInfo {
        let span = {
            let element = node.as_element();
            element.get_attr(&ns!(""), &atom!("span")).and_then(|string| {
                let n: Option<int> = FromStr::from_str(string);
                n
            }).unwrap_or(0)
        };
        TableColumnFragmentInfo {
            span: span,
        }
    }
}

impl Fragment {
    /// Constructs a new `Fragment` instance for the given node.
    ///
    /// This does *not* construct the text for generated content. See comments in
    /// `FlowConstructor::build_specific_fragment_info_for_node()` for more details.
    ///
    /// Arguments:
    ///
    ///   * `constructor`: The flow constructor.
    ///   * `node`: The node to create a fragment for.
    pub fn new(constructor: &mut FlowConstructor, node: &ThreadSafeLayoutNode) -> Fragment {
        let style = node.style().clone();
        let writing_mode = style.writing_mode;
        Fragment {
            node: OpaqueNodeMethods::from_thread_safe_layout_node(node),
            style: style,
            restyle_damage: node.restyle_damage(),
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: constructor.build_specific_fragment_info_for_node(node),
            inline_context: None,
            debug_id: layout_debug::generate_unique_debug_id(),
        }
    }

    /// Constructs a new `Fragment` instance from a specific info.
    pub fn new_from_specific_info(node: &ThreadSafeLayoutNode, specific: SpecificFragmentInfo)
                                  -> Fragment {
        let style = node.style().clone();
        let writing_mode = style.writing_mode;
        Fragment {
            node: OpaqueNodeMethods::from_thread_safe_layout_node(node),
            style: style,
            restyle_damage: node.restyle_damage(),
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: specific,
            inline_context: None,
            debug_id: layout_debug::generate_unique_debug_id(),
        }
    }

    /// Constructs a new `Fragment` instance for an anonymous table object.
    pub fn new_anonymous_table_fragment(node: &ThreadSafeLayoutNode,
                                        specific: SpecificFragmentInfo)
                                        -> Fragment {
        // CSS 2.1 § 17.2.1 This is for non-inherited properties on anonymous table fragments
        // example:
        //
        //     <div style="display: table">
        //         Foo
        //     </div>
        //
        // Anonymous table fragments, TableRowFragment and TableCellFragment, are generated around
        // `Foo`, but they shouldn't inherit the border.

        let node_style = cascade_anonymous(&**node.style());
        let writing_mode = node_style.writing_mode;
        Fragment {
            node: OpaqueNodeMethods::from_thread_safe_layout_node(node),
            style: Arc::new(node_style),
            restyle_damage: node.restyle_damage(),
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: specific,
            inline_context: None,
            debug_id: layout_debug::generate_unique_debug_id(),
        }
    }

    /// Constructs a new `Fragment` instance from an opaque node.
    pub fn from_opaque_node_and_style(node: OpaqueNode,
                                      style: Arc<ComputedValues>,
                                      restyle_damage: RestyleDamage,
                                      specific: SpecificFragmentInfo)
                                      -> Fragment {
        let writing_mode = style.writing_mode;
        Fragment {
            node: node,
            style: style,
            restyle_damage: restyle_damage,
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: specific,
            inline_context: None,
            debug_id: layout_debug::generate_unique_debug_id(),
        }
    }

    pub fn reset_inline_sizes(&mut self) {
        self.border_padding = LogicalMargin::zero(self.style.writing_mode);
        self.margin = LogicalMargin::zero(self.style.writing_mode);
    }

    /// Saves the new_line_pos vector into a `ScannedTextFragment`. This will fail
    /// if called on any other type of fragment.
    pub fn save_new_line_pos(&mut self) {
        match &mut self.specific {
            &ScannedTextFragment(ref mut info) => {
                if !info.new_line_pos.is_empty() {
                    info.original_new_line_pos = Some(info.new_line_pos.clone());
                }
            }
            _ => {}
        }
    }

    pub fn restore_new_line_pos(&mut self) {
        match &mut self.specific {
            &ScannedTextFragment(ref mut info) => {
                match info.original_new_line_pos.take() {
                    None => {}
                    Some(new_line_pos) => info.new_line_pos = new_line_pos,
                }
                return
            }
            _ => {}
        }
    }

    /// Returns a debug ID of this fragment. This ID should not be considered stable across
    /// multiple layouts or fragment manipulations.
    pub fn debug_id(&self) -> u16 {
        self.debug_id
    }

    /// Transforms this fragment into another fragment of the given type, with the given size,
    /// preserving all the other data.
    pub fn transform(&self, size: LogicalSize<Au>, mut info: Box<ScannedTextFragmentInfo>)
                     -> Fragment {
        let new_border_box = LogicalRect::from_point_size(self.style.writing_mode,
                                                          self.border_box.start,
                                                          size);

        info.content_size = size.clone();

        Fragment {
            node: self.node,
            style: self.style.clone(),
            restyle_damage: RestyleDamage::all(),
            border_box: new_border_box,
            border_padding: self.border_padding,
            margin: self.margin,
            specific: ScannedTextFragment(info),
            inline_context: self.inline_context.clone(),
            debug_id: self.debug_id,
        }
    }

    pub fn restyle_damage(&self) -> RestyleDamage {
        self.restyle_damage | self.specific.restyle_damage()
    }

    /// Adds a style to the inline context for this fragment. If the inline
    /// context doesn't exist yet, it will be created.
    pub fn add_inline_context_style(&mut self, style: Arc<ComputedValues>) {
        if self.inline_context.is_none() {
            self.inline_context = Some(InlineFragmentContext::new());
        }
        self.inline_context.as_mut().unwrap().styles.push(style.clone());
    }

    /// Determines which quantities (border/padding/margin/specified) should be included in the
    /// intrinsic inline size of this fragment.
    fn quantities_included_in_intrinsic_inline_size(&self)
                                                    -> QuantitiesIncludedInIntrinsicInlineSizes {
        match self.specific {
            GenericFragment | IframeFragment(_) | ImageFragment(_) | InlineBlockFragment(_) => {
                QuantitiesIncludedInIntrinsicInlineSizes::all()
            }
            TableFragment | TableCellFragment => {
                INTRINSIC_INLINE_SIZE_INCLUDES_PADDING |
                    INTRINSIC_INLINE_SIZE_INCLUDES_BORDER |
                    INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED
            }
            TableWrapperFragment => {
                INTRINSIC_INLINE_SIZE_INCLUDES_MARGINS |
                    INTRINSIC_INLINE_SIZE_INCLUDES_BORDER |
                    INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED
            }
            TableRowFragment => {
                INTRINSIC_INLINE_SIZE_INCLUDES_BORDER |
                    INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED
            }
            ScannedTextFragment(_) | TableColumnFragment(_) | UnscannedTextFragment(_) |
            InlineAbsoluteHypotheticalFragment(_) => {
                QuantitiesIncludedInIntrinsicInlineSizes::empty()
            }
        }
    }

    /// Returns the portion of the intrinsic inline-size that consists of borders, padding, and/or
    /// margins.
    ///
    /// FIXME(#2261, pcwalton): This won't work well for inlines: is this OK?
    pub fn surrounding_intrinsic_inline_size(&self) -> Au {
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

        margin + padding + border
    }

    /// Uses the style only to estimate the intrinsic inline-sizes. These may be modified for text
    /// or replaced elements.
    fn style_specified_intrinsic_inline_size(&self) -> IntrinsicISizesContribution {
        let flags = self.quantities_included_in_intrinsic_inline_size();
        let style = self.style();
        let specified = if flags.contains(INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED) {
            MaybeAuto::from_style(style.content_inline_size(), Au(0)).specified_or_zero()
        } else {
            Au(0)
        };

        // FIXME(#2261, pcwalton): This won't work well for inlines: is this OK?
        let surrounding_inline_size = self.surrounding_intrinsic_inline_size();

        IntrinsicISizesContribution {
            content_intrinsic_sizes: IntrinsicISizes {
                minimum_inline_size: specified,
                preferred_inline_size: specified,
            },
            surrounding_size: surrounding_inline_size,
        }
    }

    pub fn calculate_line_height(&self, layout_context: &LayoutContext) -> Au {
        let font_style = self.style.get_font_arc();
        let font_metrics = text::font_metrics_for_style(layout_context.font_context(), font_style);
        text::line_height_from_style(&*self.style, &font_metrics)
    }

    /// Returns the sum of the inline-sizes of all the borders of this fragment. Note that this
    /// can be expensive to compute, so if possible use the `border_padding` field instead.
    #[inline]
    pub fn border_width(&self) -> LogicalMargin<Au> {
        let style_border_width = match self.specific {
            ScannedTextFragment(_) => LogicalMargin::zero(self.style.writing_mode),
            _ => self.style().logical_border_width(),
        };

        match self.inline_context {
            None => style_border_width,
            Some(ref inline_fragment_context) => {
                inline_fragment_context.styles.iter().fold(style_border_width,
                                            |acc, style| acc + style.logical_border_width())
            }
        }
    }

    /// Computes the margins in the inline direction from the containing block inline-size and the
    /// style. After this call, the inline direction of the `margin` field will be correct.
    ///
    /// Do not use this method if the inline direction margins are to be computed some other way
    /// (for example, via constraint solving for blocks).
    pub fn compute_inline_direction_margins(&mut self, containing_block_inline_size: Au) {
        match self.specific {
            TableFragment | TableCellFragment | TableRowFragment | TableColumnFragment(_) => {
                self.margin.inline_start = Au(0);
                self.margin.inline_end = Au(0)
            }
            _ => {
                let margin = self.style().logical_margin();
                self.margin.inline_start =
                    MaybeAuto::from_style(margin.inline_start, containing_block_inline_size)
                    .specified_or_zero();
                self.margin.inline_end =
                    MaybeAuto::from_style(margin.inline_end, containing_block_inline_size)
                    .specified_or_zero();
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
            TableFragment | TableCellFragment | TableRowFragment | TableColumnFragment(_) => {
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
    pub fn compute_border_and_padding(&mut self, containing_block_inline_size: Au) {
        // Compute border.
        let border = self.border_width();

        // Compute padding.
        let padding = match self.specific {
            TableColumnFragment(_) | TableRowFragment |
            TableWrapperFragment => LogicalMargin::zero(self.style.writing_mode),
            _ => {
                let style_padding = match self.specific {
                    ScannedTextFragment(_) => LogicalMargin::zero(self.style.writing_mode),
                    _ => model::padding_from_style(self.style(), containing_block_inline_size),
                };

                match self.inline_context {
                    None => style_padding,
                    Some(ref inline_fragment_context) => {
                        inline_fragment_context.styles.iter().fold(style_padding,
                                |acc, style| acc + model::padding_from_style(&**style, Au(0)))
                    }
                }
            }
        };

        self.border_padding = border + padding;
    }

    // Return offset from original position because of `position: relative`.
    pub fn relative_position(&self,
                             containing_block_size: &LogicalSize<Au>)
                             -> LogicalSize<Au> {
        fn from_style(style: &ComputedValues, container_size: &LogicalSize<Au>)
                      -> LogicalSize<Au> {
            let offsets = style.logical_position();
            let offset_i = if offsets.inline_start != LPA_Auto {
                MaybeAuto::from_style(offsets.inline_start, container_size.inline).specified_or_zero()
            } else {
                -MaybeAuto::from_style(offsets.inline_end, container_size.inline).specified_or_zero()
            };
            let offset_b = if offsets.block_start != LPA_Auto {
                MaybeAuto::from_style(offsets.block_start, container_size.inline).specified_or_zero()
            } else {
                -MaybeAuto::from_style(offsets.block_end, container_size.inline).specified_or_zero()
            };
            LogicalSize::new(style.writing_mode, offset_i, offset_b)
        }

        // Go over the ancestor fragments and add all relative offsets (if any).
        let mut rel_pos = if self.style().get_box().position == position::relative {
            from_style(self.style(), containing_block_size)
        } else {
            LogicalSize::zero(self.style.writing_mode)
        };

        match self.inline_context {
            None => {}
            Some(ref inline_fragment_context) => {
                for style in inline_fragment_context.styles.iter() {
                    if style.get_box().position == position::relative {
                        rel_pos = rel_pos + from_style(&**style, containing_block_size);
                    }
                }
            },
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
            clear::none => None,
            clear::left => Some(ClearLeft),
            clear::right => Some(ClearRight),
            clear::both => Some(ClearBoth),
        }
    }

    #[inline(always)]
    pub fn style<'a>(&'a self) -> &'a ComputedValues {
        &*self.style
    }

    /// Returns the text alignment of the computed style of the nearest ancestor-or-self `Element`
    /// node.
    pub fn text_align(&self) -> text_align::T {
        self.style().get_inheritedtext().text_align
    }

    pub fn vertical_align(&self) -> vertical_align::T {
        self.style().get_box().vertical_align
    }

    pub fn white_space(&self) -> white_space::T {
        self.style().get_inheritedtext().white_space
    }

    /// Returns the text decoration of this fragment, according to the style of the nearest ancestor
    /// element.
    ///
    /// NB: This may not be the actual text decoration, because of the override rules specified in
    /// CSS 2.1 § 16.3.1. Unfortunately, computing this properly doesn't really fit into Servo's
    /// model. Therefore, this is a best lower bound approximation, but the end result may actually
    /// have the various decoration flags turned on afterward.
    pub fn text_decoration(&self) -> text_decoration::T {
        self.style().get_text().text_decoration
    }

    /// Returns the inline-start offset from margin edge to content edge.
    ///
    /// FIXME(#2262, pcwalton): I think this method is pretty bogus, because it won't work for
    /// inlines.
    pub fn inline_start_offset(&self) -> Au {
        match self.specific {
            TableWrapperFragment => self.margin.inline_start,
            TableFragment | TableCellFragment | TableRowFragment => self.border_padding.inline_start,
            TableColumnFragment(_) => Au(0),
            _ => self.margin.inline_start + self.border_padding.inline_start,
        }
    }

    /// Returns true if this element can be split. This is true for text fragments.
    pub fn can_split(&self) -> bool {
        self.is_scanned_text_fragment()
    }

    /// Returns the newline positions of this fragment, if it's a scanned text fragment.
    pub fn newline_positions(&self) -> Option<&Vec<CharIndex>> {
        match self.specific {
            ScannedTextFragment(ref info) => Some(&info.new_line_pos),
            _ => None,
        }
    }

    /// Returns the newline positions of this fragment, if it's a scanned text fragment.
    pub fn newline_positions_mut(&mut self) -> Option<&mut Vec<CharIndex>> {
        match self.specific {
            ScannedTextFragment(ref mut info) => Some(&mut info.new_line_pos),
            _ => None,
        }
    }

    /// Returns true if and only if this is a scanned text fragment.
    fn is_scanned_text_fragment(&self) -> bool {
        match self.specific {
            ScannedTextFragment(..) => true,
            _ => false,
        }
    }

    /// Computes the intrinsic inline-sizes of this fragment.
    pub fn compute_intrinsic_inline_sizes(&mut self) -> IntrinsicISizesContribution {
        let mut result = self.style_specified_intrinsic_inline_size();
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment |
            TableColumnFragment(_) | TableRowFragment | TableWrapperFragment |
            InlineAbsoluteHypotheticalFragment(_) => {}
            InlineBlockFragment(ref mut info) => {
                let block_flow = info.flow_ref.as_block();
                result.union_block(&block_flow.base.intrinsic_inline_sizes)
            }
            ImageFragment(ref mut image_fragment_info) => {
                let image_inline_size = image_fragment_info.image_inline_size();
                result.union_block(&IntrinsicISizes {
                    minimum_inline_size: image_inline_size,
                    preferred_inline_size: image_inline_size,
                })
            }
            ScannedTextFragment(ref text_fragment_info) => {
                let range = &text_fragment_info.range;
                let min_line_inline_size = text_fragment_info.run.min_width_for_range(range);

                // See http://dev.w3.org/csswg/css-sizing/#max-content-inline-size.
                // TODO: Account for soft wrap opportunities.
                let max_line_inline_size = text_fragment_info.run
                                                             .metrics_for_range(range)
                                                             .advance_width;

                result.union_block(&IntrinsicISizes {
                    minimum_inline_size: min_line_inline_size,
                    preferred_inline_size: max_line_inline_size,
                })
            }
            UnscannedTextFragment(..) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            }
        };

        // Take borders and padding for parent inline fragments into account, if necessary.
        if self.is_primary_fragment() {
            match self.inline_context {
                None => {}
                Some(ref context) => {
                    for style in context.styles.iter() {
                        let border_width = style.logical_border_width().inline_start_end();
                        let padding_inline_size =
                            model::padding_from_style(&**style, Au(0)).inline_start_end();
                        result.surrounding_size = result.surrounding_size + border_width +
                            padding_inline_size;
                    }
                }
            }
        }

        result
    }


    /// TODO: What exactly does this function return? Why is it Au(0) for GenericFragment?
    pub fn content_inline_size(&self) -> Au {
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment |
            TableRowFragment | TableWrapperFragment | InlineBlockFragment(_) |
            InlineAbsoluteHypotheticalFragment(_) => Au(0),
            ImageFragment(ref image_fragment_info) => {
                image_fragment_info.computed_inline_size()
            }
            ScannedTextFragment(ref text_fragment_info) => {
                let (range, run) = (&text_fragment_info.range, &text_fragment_info.run);
                let text_bounds = run.metrics_for_range(range).bounding_box;
                text_bounds.size.width
            }
            TableColumnFragment(_) => panic!("Table column fragments do not have inline_size"),
            UnscannedTextFragment(_) => panic!("Unscanned text fragments should have been scanned by now!"),
        }
    }

    /// Returns, and computes, the block-size of this fragment.
    pub fn content_block_size(&self, layout_context: &LayoutContext) -> Au {
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment |
            TableRowFragment | TableWrapperFragment | InlineBlockFragment(_) |
            InlineAbsoluteHypotheticalFragment(_) => Au(0),
            ImageFragment(ref image_fragment_info) => {
                image_fragment_info.computed_block_size()
            }
            ScannedTextFragment(_) => {
                // Compute the block-size based on the line-block-size and font size.
                self.calculate_line_height(layout_context)
            }
            TableColumnFragment(_) => panic!("Table column fragments do not have block_size"),
            UnscannedTextFragment(_) => panic!("Unscanned text fragments should have been scanned by now!"),
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

    /// Find the split of a fragment that includes a new-line character.
    ///
    /// A return value of `None` indicates that the fragment is not splittable.
    /// Otherwise the split information is returned. The right information is
    /// optional due to the possibility of it being whitespace.
    //
    // TODO(bjz): The text run should be removed in the future, but it is currently needed for
    // the current method of fragment splitting in the `inline::try_append_*` functions.
    pub fn find_split_info_by_new_line(&self)
            -> Option<(SplitInfo, Option<SplitInfo>, Arc<Box<TextRun>> /* TODO(bjz): remove */)> {
        match self.specific {
            GenericFragment | IframeFragment(_) | ImageFragment(_) | TableFragment | TableCellFragment |
            TableRowFragment | TableWrapperFragment => None,
            TableColumnFragment(_) => panic!("Table column fragments do not need to split"),
            UnscannedTextFragment(_) => panic!("Unscanned text fragments should have been scanned by now!"),
            InlineBlockFragment(_) | InlineAbsoluteHypotheticalFragment(_) => {
                panic!("Inline blocks or inline absolute hypothetical fragments do not get split")
            }
            ScannedTextFragment(ref text_fragment_info) => {
                let mut new_line_pos = text_fragment_info.new_line_pos.clone();
                let cur_new_line_pos = new_line_pos.remove(0).unwrap();

                let inline_start_range = Range::new(text_fragment_info.range.begin(),
                                                    cur_new_line_pos);
                let inline_end_range = Range::new(
                    text_fragment_info.range.begin() + cur_new_line_pos + CharIndex(1),
                    text_fragment_info.range.length() - (cur_new_line_pos + CharIndex(1)));

                // Left fragment is for inline-start text of first founded new-line character.
                let inline_start_fragment = SplitInfo::new(inline_start_range,
                                                           &**text_fragment_info);

                // Right fragment is for inline-end text of first founded new-line character.
                let inline_end_fragment = if inline_end_range.length() > CharIndex(0) {
                    Some(SplitInfo::new(inline_end_range, &**text_fragment_info))
                } else {
                    None
                };

                Some((inline_start_fragment, inline_end_fragment, text_fragment_info.run.clone()))
            }
        }
    }

    /// Attempts to find the split positions of a text fragment so that its inline-size is
    /// no more than `max_inline-size`.
    ///
    /// A return value of `None` indicates that the fragment could not be split.
    /// Otherwise the information pertaining to the split is returned. The inline-start
    /// and inline-end split information are both optional due to the possibility of
    /// them being whitespace.
    pub fn find_split_info_for_inline_size(&self,
                                           start: CharIndex,
                                           max_inline_size: Au,
                                           starts_line: bool)
                                           -> Option<(Option<SplitInfo>,
                                                      Option<SplitInfo>,
                                                      Arc<Box<TextRun>>)> {
        match self.specific {
            GenericFragment | IframeFragment(_) | ImageFragment(_) | TableFragment |
            TableCellFragment | TableRowFragment | TableWrapperFragment | InlineBlockFragment(_) |
            InlineAbsoluteHypotheticalFragment(_) => None,
            TableColumnFragment(_) => panic!("Table column fragments do not have inline_size"),
            UnscannedTextFragment(_) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            }
            ScannedTextFragment(ref text_fragment_info) => {
                let mut pieces_processed_count: uint = 0;
                let mut remaining_inline_size: Au = max_inline_size;
                let mut inline_start_range = Range::new(text_fragment_info.range.begin() + start,
                                                        CharIndex(0));
                let mut inline_end_range: Option<Range<CharIndex>> = None;

                debug!("split_to_inline_size: splitting text fragment \
                        (strlen={}, range={}, avail_inline_size={})",
                       text_fragment_info.run.text.len(),
                       text_fragment_info.range,
                       max_inline_size);

                for (glyphs, offset, slice_range) in text_fragment_info.run.iter_slices_for_range(
                        &text_fragment_info.range) {
                    debug!("split_to_inline_size: considering slice (offset={}, range={}, \
                                                               remain_inline_size={})",
                           offset,
                           slice_range,
                           remaining_inline_size);

                    let metrics = text_fragment_info.run.metrics_for_slice(glyphs, &slice_range);
                    let advance = metrics.advance_width;

                    let should_continue;
                    if advance <= remaining_inline_size || glyphs.is_whitespace() {
                        should_continue = true;

                        if starts_line && pieces_processed_count == 0 && glyphs.is_whitespace() {
                            debug!("split_to_inline_size: case=skipping leading trimmable whitespace");
                            inline_start_range.shift_by(slice_range.length());
                        } else {
                            debug!("split_to_inline_size: case=enlarging span");
                            remaining_inline_size = remaining_inline_size - advance;
                            inline_start_range.extend_by(slice_range.length());
                        }
                    } else {
                        // The advance is more than the remaining inline-size.
                        should_continue = false;
                        let slice_begin = offset + slice_range.begin();

                        if slice_begin < text_fragment_info.range.end() {
                            // There are still some things inline-start over at the end of the line. Create
                            // the inline-end chunk.
                            let inline_end_range_end = text_fragment_info.range.end() - slice_begin;
                            inline_end_range = Some(Range::new(slice_begin, inline_end_range_end));
                            debug!("split_to_inline_size: case=splitting remainder with inline_end range={}",
                                   inline_end_range);
                        }
                    }

                    pieces_processed_count += 1;

                    if !should_continue {
                        break
                    }
                }

                let inline_start_is_some = inline_start_range.length() > CharIndex(0);

                if (pieces_processed_count == 1 || !inline_start_is_some) && !starts_line {
                    None
                } else {
                    let inline_start = if inline_start_is_some {
                        Some(SplitInfo::new(inline_start_range, &**text_fragment_info))
                    } else {
                         None
                    };
                    let inline_end = inline_end_range.map(|inline_end_range| {
                        SplitInfo::new(inline_end_range, &**text_fragment_info)
                    });

                    Some((inline_start, inline_end, text_fragment_info.run.clone()))
                }
            }
        }
    }

    /// Returns true if this fragment is an unscanned text fragment that consists entirely of
    /// whitespace that should be stripped.
    pub fn is_ignorable_whitespace(&self) -> bool {
        match self.white_space() {
            white_space::pre => return false,
            white_space::normal | white_space::nowrap => {}
        }
        match self.specific {
            UnscannedTextFragment(ref text_fragment_info) => {
                is_whitespace(text_fragment_info.text.as_slice())
            }
            _ => false,
        }
    }

    /// Assigns replaced inline-size, padding, and margins for this fragment only if it is replaced
    /// content per CSS 2.1 § 10.3.2.
    pub fn assign_replaced_inline_size_if_necessary(&mut self, container_inline_size: Au) {
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment |
            TableRowFragment | TableWrapperFragment => return,
            TableColumnFragment(_) => panic!("Table column fragments do not have inline_size"),
            UnscannedTextFragment(_) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            }
            ImageFragment(_) | ScannedTextFragment(_) | InlineBlockFragment(_) |
            InlineAbsoluteHypotheticalFragment(_) => {}
        };

        let style_inline_size = self.style().content_inline_size();
        let style_block_size = self.style().content_block_size();
        let style_min_inline_size = self.style().min_inline_size();
        let style_max_inline_size = self.style().max_inline_size();
        let style_min_block_size = self.style().min_block_size();
        let style_max_block_size = self.style().max_block_size();
        let noncontent_inline_size = self.border_padding.inline_start_end();

        match self.specific {
            InlineAbsoluteHypotheticalFragment(ref mut info) => {
                let block_flow = info.flow_ref.as_block();
                block_flow.base.position.size.inline =
                    block_flow.base.intrinsic_inline_sizes.preferred_inline_size;

                // This is a hypothetical box, so it takes up no space.
                self.border_box.size.inline = Au(0);
            }
            InlineBlockFragment(ref mut info) => {
                let block_flow = info.flow_ref.as_block();
                self.border_box.size.inline =
                    block_flow.base.intrinsic_inline_sizes.preferred_inline_size;
                block_flow.base.block_container_inline_size = self.border_box.size.inline;
            }
            ScannedTextFragment(ref info) => {
                // Scanned text fragments will have already had their content inline-sizes assigned
                // by this point.
                self.border_box.size.inline = info.content_size.inline + noncontent_inline_size
            }
            ImageFragment(ref mut image_fragment_info) => {
                // TODO(ksh8281): compute border,margin
                let inline_size = ImageFragmentInfo::style_length(
                    style_inline_size,
                    image_fragment_info.dom_inline_size,
                    container_inline_size);

                let inline_size = match inline_size {
                    Auto => {
                        let intrinsic_width = image_fragment_info.image_inline_size();
                        let intrinsic_height = image_fragment_info.image_block_size();

                        if intrinsic_height == Au(0) {
                            intrinsic_width
                        } else {
                            let ratio = intrinsic_width.to_f32().unwrap() /
                                        intrinsic_height.to_f32().unwrap();

                            let specified_height = ImageFragmentInfo::style_length(
                                style_block_size,
                                image_fragment_info.dom_block_size,
                                Au(0));
                            let specified_height = match specified_height {
                                Auto => intrinsic_height,
                                Specified(h) => h,
                            };
                            let specified_height = ImageFragmentInfo::clamp_size(
                                specified_height,
                                style_min_block_size,
                                style_max_block_size,
                                Au(0));
                            Au((specified_height.to_f32().unwrap() * ratio) as i32)
                        }
                    },
                    Specified(w) => w,
                };

                let inline_size = ImageFragmentInfo::clamp_size(inline_size,
                                                                style_min_inline_size,
                                                                style_max_inline_size,
                                                                container_inline_size);

                self.border_box.size.inline = inline_size + noncontent_inline_size;
                image_fragment_info.computed_inline_size = Some(inline_size);
            }
            _ => panic!("this case should have been handled above"),
        }
    }

    /// Assign block-size for this fragment if it is replaced content. The inline-size must have
    /// been assigned first.
    ///
    /// Ideally, this should follow CSS 2.1 § 10.6.2.
    pub fn assign_replaced_block_size_if_necessary(&mut self, containing_block_block_size: Au) {
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment |
            TableRowFragment | TableWrapperFragment => return,
            TableColumnFragment(_) => panic!("Table column fragments do not have block_size"),
            UnscannedTextFragment(_) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            }
            ImageFragment(_) | ScannedTextFragment(_) | InlineBlockFragment(_) |
            InlineAbsoluteHypotheticalFragment(_) => {}
        }

        let style_block_size = self.style().content_block_size();
        let style_min_block_size = self.style().min_block_size();
        let style_max_block_size = self.style().max_block_size();
        let noncontent_block_size = self.border_padding.block_start_end();

        match self.specific {
            ImageFragment(ref mut image_fragment_info) => {
                // TODO(ksh8281): compute border,margin,padding
                let inline_size = image_fragment_info.computed_inline_size();
                let block_size = ImageFragmentInfo::style_length(
                    style_block_size,
                    image_fragment_info.dom_block_size,
                    containing_block_block_size);

                let block_size = match block_size {
                    Auto => {
                        let scale = image_fragment_info.image_inline_size().to_f32().unwrap()
                            / inline_size.to_f32().unwrap();
                        Au((image_fragment_info.image_block_size().to_f32().unwrap() / scale)
                           as i32)
                    },
                    Specified(h) => {
                        h
                    }
                };

                let block_size = ImageFragmentInfo::clamp_size(block_size, style_min_block_size,
                                                               style_max_block_size,
                                                               Au(0));

                image_fragment_info.computed_block_size = Some(block_size);
                self.border_box.size.block = block_size + noncontent_block_size
            }
            ScannedTextFragment(ref info) => {
                // Scanned text fragments' content block-sizes are calculated by the text run
                // scanner during flow construction.
                self.border_box.size.block = info.content_size.block + noncontent_block_size
            }
            InlineBlockFragment(ref mut info) => {
                // Not the primary fragment, so we do not take the noncontent size into account.
                let block_flow = info.flow_ref.as_block();
                self.border_box.size.block = block_flow.base.position.size.block +
                    block_flow.fragment.margin.block_start_end()
            }
            InlineAbsoluteHypotheticalFragment(ref mut info) => {
                // Not the primary fragment, so we do not take the noncontent size into account.
                let block_flow = info.flow_ref.as_block();
                self.border_box.size.block = block_flow.base.position.size.block;
            }
            _ => panic!("should have been handled above"),
        }
    }

    /// Calculates block-size above baseline, depth below baseline, and ascent for this fragment when
    /// used in an inline formatting context. See CSS 2.1 § 10.8.1.
    pub fn inline_metrics(&self, layout_context: &LayoutContext) -> InlineMetrics {
        match self.specific {
            ImageFragment(ref image_fragment_info) => {
                let computed_block_size = image_fragment_info.computed_block_size();
                InlineMetrics {
                    block_size_above_baseline: computed_block_size + self.border_padding.block_start_end(),
                    depth_below_baseline: Au(0),
                    ascent: computed_block_size + self.border_padding.block_end,
                }
            }
            ScannedTextFragment(ref text_fragment) => {
                // See CSS 2.1 § 10.8.1.
                let line_height = self.calculate_line_height(layout_context);
                InlineMetrics::from_font_metrics(&text_fragment.run.font_metrics, line_height)
            }
            InlineBlockFragment(ref info) => {
                // See CSS 2.1 § 10.8.1.
                let block_flow = info.flow_ref.deref().as_immutable_block();
                let font_style = self.style.get_font_arc();
                let font_metrics = text::font_metrics_for_style(layout_context.font_context(),
                                                                font_style);
                InlineMetrics::from_block_height(&font_metrics,
                                                 block_flow.base.position.size.block +
                                                 block_flow.fragment.margin.block_start_end())
            }
            InlineAbsoluteHypotheticalFragment(_) => {
                // Hypothetical boxes take up no space.
                InlineMetrics {
                    block_size_above_baseline: Au(0),
                    depth_below_baseline: Au(0),
                    ascent: Au(0),
                }
            }
            _ => {
                InlineMetrics {
                    block_size_above_baseline: self.border_box.size.block,
                    depth_below_baseline: Au(0),
                    ascent: self.border_box.size.block,
                }
            }
        }
    }

    /// Returns true if this fragment is a hypothetical box. See CSS 2.1 § 10.3.7.
    pub fn is_hypothetical(&self) -> bool {
        match self.specific {
            InlineAbsoluteHypotheticalFragment(_) => true,
            _ => false,
        }
    }

    /// Returns true if this fragment can merge with another adjacent fragment or false otherwise.
    pub fn can_merge_with_fragment(&self, other: &Fragment) -> bool {
        match (&self.specific, &other.specific) {
            (&UnscannedTextFragment(_), &UnscannedTextFragment(_)) => {
                // FIXME: Should probably use a whitelist of styles that can safely differ (#3165)
                self.style().get_font() == other.style().get_font() &&
                    self.text_decoration() == other.text_decoration() &&
                    self.white_space() == other.white_space()
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
            InlineBlockFragment(_) | InlineAbsoluteHypotheticalFragment(_) |
            TableWrapperFragment => false,
            GenericFragment | IframeFragment(_) | ImageFragment(_) | ScannedTextFragment(_) |
            TableFragment | TableCellFragment | TableColumnFragment(_) | TableRowFragment |
            UnscannedTextFragment(_) => true,
        }
    }

    pub fn update_late_computed_inline_position_if_necessary(&mut self) {
        match self.specific {
            InlineAbsoluteHypotheticalFragment(ref mut info) => {
                let position = self.border_box.start.i;
                info.flow_ref.update_late_computed_inline_position_if_necessary(position)
            }
            _ => {}
        }
    }

    pub fn update_late_computed_block_position_if_necessary(&mut self) {
        match self.specific {
            InlineAbsoluteHypotheticalFragment(ref mut info) => {
                let position = self.border_box.start.b;
                info.flow_ref.update_late_computed_block_position_if_necessary(position)
            }
            _ => {}
        }
    }

    pub fn repair_style(&mut self, new_style: &Arc<ComputedValues>) {
        self.style = (*new_style).clone()
    }

    /// Given the stacking-context-relative position of the containing flow, returns the boundaries
    /// of this fragment relative to the parent stacking context.
    pub fn stacking_relative_bounds(&self, stacking_relative_flow_origin: &Point2D<Au>)
                                    -> Rect<Au> {
        // FIXME(#2795): Get the real container size
        let container_size = Size2D::zero();
        self.border_box
            .to_physical(self.style.writing_mode, container_size)
            .translate(stacking_relative_flow_origin)
    }

    /// Returns true if this fragment establishes a new stacking context and false otherwise.
    pub fn establishes_stacking_context(&self) -> bool {
        match self.style().get_box().position {
            position::absolute | position::fixed => {
                // FIXME(pcwalton): This should only establish a new stacking context when
                // `z-index` is not `auto`. But this matches what we did before.
                true
            }
            position::relative | position::static_ => {
                // FIXME(pcwalton): `position: relative` establishes a new stacking context if
                // `z-index` is not `auto`. But this matches what we did before.
                false
            }
        }
    }
}

impl fmt::Show for Fragment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "({} {} ", self.debug_id(), self.specific.get_type()));
        try!(write!(f, "bp {}", self.border_padding));
        try!(write!(f, " "));
        try!(write!(f, "m {}", self.margin));
        write!(f, ")")
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

/// A top-down fragment bounds iteration handler.
pub trait FragmentBoundsIterator {
    /// The operation to perform.
    fn process(&mut self, fragment: &Fragment, bounds: Rect<Au>);

    /// Returns true if this fragment must be processed in-order. If this returns false,
    /// we skip the operation for this fragment, but continue processing siblings.
    fn should_process(&mut self, fragment: &Fragment) -> bool;
}
