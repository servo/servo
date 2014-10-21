/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Fragment` type, which represents the leaves of the layout tree.

#![deny(unsafe_block)]

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
use util::{OpaqueNodeMethods, ToGfxColor};
use wrapper::{TLayoutNode, ThreadSafeLayoutNode};

use geom::{Point2D, Rect, Size2D, SideOffsets2D};
use geom::approxeq::ApproxEq;
use gfx::color::rgb;
use gfx::display_list::{BackgroundAndBorderLevel, BaseDisplayItem, BorderDisplayItem};
use gfx::display_list::{BorderDisplayItemClass, ContentStackingLevel, DisplayList};
use gfx::display_list::{ImageDisplayItem, ImageDisplayItemClass, LineDisplayItem};
use gfx::display_list::{LineDisplayItemClass, OpaqueNode, PseudoDisplayItemClass};
use gfx::display_list::{SidewaysLeft, SidewaysRight, SolidColorDisplayItem};
use gfx::display_list::{SolidColorDisplayItemClass, StackingLevel, TextDisplayItem};
use gfx::display_list::{TextDisplayItemClass, Upright};
use gfx::text::glyph::CharIndex;
use gfx::text::text_run::TextRun;
use script_traits::UntrustedNodeAddress;
use serialize::{Encodable, Encoder};
use servo_msg::constellation_msg::{ConstellationChan, FrameRectMsg, PipelineId, SubpageId};
use servo_net::image::holder::ImageHolder;
use servo_net::local_image_cache::LocalImageCache;
use servo_util::geometry::{Au, ZERO_RECT};
use servo_util::geometry;
use servo_util::logical_geometry::{LogicalRect, LogicalSize, LogicalMargin, WritingMode};
use servo_util::opts;
use servo_util::range::*;
use servo_util::smallvec::SmallVec;
use servo_util::str::is_whitespace;
use std::cmp::{max, min};
use std::fmt;
use std::from_str::FromStr;
use std::num::Zero;
use string_cache::Atom;
use style::{ComputedValues, TElement, TNode, cascade_anonymous, RGBA};
use style::computed_values::{LengthOrPercentage, LengthOrPercentageOrAuto};
use style::computed_values::{LengthOrPercentageOrNone};
use style::computed_values::{overflow, LPA_Auto, background_attachment};
use style::computed_values::{background_repeat, border_style, clear, position, text_align};
use style::computed_values::{text_decoration, vertical_align, visibility, white_space};
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
/// FIXME(#2260, pcwalton): This can be slimmed down some.
#[deriving(Clone)]
pub struct Fragment {
    /// An opaque reference to the DOM node that this `Fragment` originates from.
    pub node: OpaqueNode,

    /// The CSS style of this fragment.
    pub style: Arc<ComputedValues>,

    /// How damaged this fragment is since last reflow.
    pub restyle_damage: RestyleDamage,

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

    /// New-line chracter(\n)'s positions(relative, not absolute)
    ///
    /// FIXME(#2260, pcwalton): This is very inefficient; remove.
    pub new_line_pos: Vec<CharIndex>,

    /// Holds the style context information for fragments
    /// that are part of an inline formatting context.
    pub inline_context: Option<InlineFragmentContext>,

    /// A debug ID that is consistent for the life of
    /// this fragment (via transform etc).
    pub debug_id: uint,
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

/// Info specific to the kind of fragment. Keep this enum small.
#[deriving(Clone)]
pub enum SpecificFragmentInfo {
    GenericFragment,
    IframeFragment(IframeFragmentInfo),
    ImageFragment(ImageFragmentInfo),

    /// A hypothetical box (see CSS 2.1 § 10.3.7) for an absolutely-positioned block that was
    /// declared with `display: inline;`.
    InlineAbsoluteHypotheticalFragment(InlineAbsoluteHypotheticalFragmentInfo),

    InlineBlockFragment(InlineBlockFragmentInfo),
    InputFragment,
    ScannedTextFragment(ScannedTextFragmentInfo),
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
                | InputFragment
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
            InputFragment => "InputFragment",
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

    /// The new_line_pos is eaten during line breaking. If we need to re-merge
    /// fragments, it will have to be restored.
    pub original_new_line_pos: Option<Vec<CharIndex>>,

    /// The inline-size of the text fragment.
    pub content_inline_size: Au,
}

impl ScannedTextFragmentInfo {
    /// Creates the information specific to a scanned text fragment from a range and a text run.
    pub fn new(run: Arc<Box<TextRun>>, range: Range<CharIndex>, content_inline_size: Au)
               -> ScannedTextFragmentInfo {
        ScannedTextFragmentInfo {
            run: run,
            range: range,
            original_new_line_pos: None,
            content_inline_size: content_inline_size,
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

/// Data for an unscanned text fragment. Unscanned text fragments are the results of flow construction that
/// have not yet had their inline-size determined.
#[deriving(Clone)]
pub struct UnscannedTextFragmentInfo {
    /// The text inside the fragment.
    pub text: String,
}

impl UnscannedTextFragmentInfo {
    /// Creates a new instance of `UnscannedTextFragmentInfo` from the given DOM node.
    pub fn new(node: &ThreadSafeLayoutNode) -> UnscannedTextFragmentInfo {
        // FIXME(pcwalton): Don't copy text; atomically reference count it instead.
        UnscannedTextFragmentInfo {
            text: node.text(),
        }
    }

    /// Creates a new instance of `UnscannedTextFragmentInfo` from the given text.
    #[inline]
    pub fn from_text(text: String) -> UnscannedTextFragmentInfo {
        UnscannedTextFragmentInfo {
            text: text,
        }
    }
}

/// A fragment that represents a table column.
#[deriving(Clone)]
pub struct TableColumnFragmentInfo {
    /// the number of columns a <col> element should span
    pub span: Option<int>,
}

impl TableColumnFragmentInfo {
    /// Create the information specific to an table column fragment.
    pub fn new(node: &ThreadSafeLayoutNode) -> TableColumnFragmentInfo {
        let span = {
            let element = node.as_element();
            element.get_attr(&ns!(""), &atom!("span")).and_then(|string| {
                let n: Option<int> = FromStr::from_str(string);
                n
            })
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
            new_line_pos: vec!(),
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
            new_line_pos: vec!(),
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
            new_line_pos: vec!(),
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
            new_line_pos: vec!(),
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
                if !self.new_line_pos.is_empty() {
                    info.original_new_line_pos = Some(self.new_line_pos.clone());
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
                    Some(new_line_pos) => self.new_line_pos = new_line_pos,
                }
                return
            }
            _ => {}
        }
    }

    /// Returns a debug ID of this fragment. This ID should not be considered stable across
    /// multiple layouts or fragment manipulations.
    pub fn debug_id(&self) -> uint {
        self.debug_id
    }

    /// Transforms this fragment into another fragment of the given type, with the given size,
    /// preserving all the other data.
    pub fn transform(&self, size: LogicalSize<Au>, mut info: ScannedTextFragmentInfo) -> Fragment {
        let new_border_box =
            LogicalRect::from_point_size(self.style.writing_mode, self.border_box.start, size);

        info.content_inline_size = size.inline;

        Fragment {
            node: self.node,
            style: self.style.clone(),
            restyle_damage: RestyleDamage::all(),
            border_box: new_border_box,
            border_padding: self.border_padding,
            margin: self.margin,
            specific: ScannedTextFragment(info),
            new_line_pos: self.new_line_pos.clone(),
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
            GenericFragment | IframeFragment(_) | ImageFragment(_) | InlineBlockFragment(_) |
            InputFragment => QuantitiesIncludedInIntrinsicInlineSizes::all(),
            TableFragment | TableCellFragment => {
                IntrinsicInlineSizeIncludesPadding |
                    IntrinsicInlineSizeIncludesBorder |
                    IntrinsicInlineSizeIncludesSpecified
            }
            TableWrapperFragment => {
                IntrinsicInlineSizeIncludesMargins |
                    IntrinsicInlineSizeIncludesBorder |
                    IntrinsicInlineSizeIncludesSpecified
            }
            TableRowFragment => {
                IntrinsicInlineSizeIncludesBorder |
                    IntrinsicInlineSizeIncludesSpecified
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
        let margin = if flags.contains(IntrinsicInlineSizeIncludesMargins) {
            let margin = style.logical_margin();
            (MaybeAuto::from_style(margin.inline_start, Au(0)).specified_or_zero() +
             MaybeAuto::from_style(margin.inline_end, Au(0)).specified_or_zero())
        } else {
            Au(0)
        };

        // FIXME(pcwalton): Percentages should be relative to any definite size per CSS-SIZING.
        // This will likely need to be done by pushing down definite sizes during selector
        // cascading.
        let padding = if flags.contains(IntrinsicInlineSizeIncludesPadding) {
            let padding = style.logical_padding();
            (model::specified(padding.inline_start, Au(0)) +
             model::specified(padding.inline_end, Au(0)))
        } else {
            Au(0)
        };

        let border = if flags.contains(IntrinsicInlineSizeIncludesBorder) {
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
        let specified = if flags.contains(IntrinsicInlineSizeIncludesSpecified) {
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
        let font_style = self.style.get_font();
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

        self.border_padding = border + padding
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

    /// Returns true if and only if this is a scanned text fragment.
    fn is_scanned_text_fragment(&self) -> bool {
        match self.specific {
            ScannedTextFragment(..) => true,
            _ => false,
        }
    }

    /// Adds the display items necessary to paint the background of this fragment to the display
    /// list if necessary.
    pub fn build_display_list_for_background_if_applicable(&self,
                                                           style: &ComputedValues,
                                                           list: &mut DisplayList,
                                                           layout_context: &LayoutContext,
                                                           level: StackingLevel,
                                                           absolute_bounds: &Rect<Au>,
                                                           clip_rect: &Rect<Au>) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a fragment".
        let background_color = style.resolve_color(style.get_background().background_color);
        if !background_color.alpha.approx_eq(&0.0) {
            let display_item = box SolidColorDisplayItem {
                base: BaseDisplayItem::new(*absolute_bounds, self.node, level, *clip_rect),
                color: background_color.to_gfx_color(),
            };

            list.push(SolidColorDisplayItemClass(display_item))
        }

        // The background image is painted on top of the background color.
        // Implements background image, per spec:
        // http://www.w3.org/TR/CSS21/colors.html#background
        let background = style.get_background();
        let image_url = match background.background_image {
            None => return,
            Some(ref image_url) => image_url,
        };

        let mut holder = ImageHolder::new(image_url.clone(), layout_context.shared.image_cache.clone());
        let image = match holder.get_image(self.node.to_untrusted_node_address()) {
            None => {
                // No image data at all? Do nothing.
                //
                // TODO: Add some kind of placeholder background image.
                debug!("(building display list) no background image :(");
                return
            }
            Some(image) => image,
        };
        debug!("(building display list) building background image");

        let image_width = Au::from_px(image.width as int);
        let image_height = Au::from_px(image.height as int);
        let mut bounds = *absolute_bounds;

        // Clip.
        //
        // TODO: Check the bounds to see if a clip item is actually required.
        let clip_rect = clip_rect.intersection(&bounds).unwrap_or(ZERO_RECT);

        // Use background-attachment to get the initial virtual origin
        let (virtual_origin_x, virtual_origin_y) = match background.background_attachment {
            background_attachment::scroll => {
                (absolute_bounds.origin.x, absolute_bounds.origin.y)
            }
            background_attachment::fixed => {
                (Au(0), Au(0))
            }
        };

        // Use background-position to get the offset
        let horizontal_position = model::specified(background.background_position.horizontal,
                                                   bounds.size.width - image_width);
        let vertical_position = model::specified(background.background_position.vertical,
                                                 bounds.size.height - image_height);

        let abs_x = virtual_origin_x + horizontal_position;
        let abs_y = virtual_origin_y + vertical_position;

        // Adjust origin and size based on background-repeat
        match background.background_repeat {
            background_repeat::no_repeat => {
                bounds.origin.x = abs_x;
                bounds.origin.y = abs_y;
                bounds.size.width = image_width;
                bounds.size.height = image_height;
            }
            background_repeat::repeat_x => {
                bounds.origin.y = abs_y;
                bounds.size.height = image_height;
                ImageFragmentInfo::tile_image(&mut bounds.origin.x, &mut bounds.size.width,
                                                abs_x, image.width);
            }
            background_repeat::repeat_y => {
                bounds.origin.x = abs_x;
                bounds.size.width = image_width;
                ImageFragmentInfo::tile_image(&mut bounds.origin.y, &mut bounds.size.height,
                                                abs_y, image.height);
            }
            background_repeat::repeat => {
                ImageFragmentInfo::tile_image(&mut bounds.origin.x, &mut bounds.size.width,
                                                abs_x, image.width);
                ImageFragmentInfo::tile_image(&mut bounds.origin.y, &mut bounds.size.height,
                                                abs_y, image.height);
            }
        };

        // Create the image display item.
        let image_display_item = ImageDisplayItemClass(box ImageDisplayItem {
            base: BaseDisplayItem::new(bounds, self.node, level, clip_rect),
            image: image.clone(),
            stretch_size: Size2D(Au::from_px(image.width as int),
                                 Au::from_px(image.height as int)),
        });
        list.push(image_display_item)
    }

    /// Adds the display items necessary to paint the borders of this fragment to a display list if
    /// necessary.
    pub fn build_display_list_for_borders_if_applicable(&self,
                                                        style: &ComputedValues,
                                                        list: &mut DisplayList,
                                                        abs_bounds: &Rect<Au>,
                                                        level: StackingLevel,
                                                        clip_rect: &Rect<Au>) {
        let border = style.logical_border_width();
        if border.is_zero() {
            return
        }

        let top_color = style.resolve_color(style.get_border().border_top_color);
        let right_color = style.resolve_color(style.get_border().border_right_color);
        let bottom_color = style.resolve_color(style.get_border().border_bottom_color);
        let left_color = style.resolve_color(style.get_border().border_left_color);

        // Append the border to the display list.
        let border_display_item = box BorderDisplayItem {
            base: BaseDisplayItem::new(*abs_bounds, self.node, level, *clip_rect),
            border: border.to_physical(style.writing_mode),
            color: SideOffsets2D::new(top_color.to_gfx_color(),
                                      right_color.to_gfx_color(),
                                      bottom_color.to_gfx_color(),
                                      left_color.to_gfx_color()),
            style: SideOffsets2D::new(style.get_border().border_top_style,
                                      style.get_border().border_right_style,
                                      style.get_border().border_bottom_style,
                                      style.get_border().border_left_style)
        };

        list.push(BorderDisplayItemClass(border_display_item))
    }

    fn build_debug_borders_around_text_fragments(&self,
                                                 display_list: &mut DisplayList,
                                                 flow_origin: Point2D<Au>,
                                                 text_fragment: &ScannedTextFragmentInfo,
                                                 clip_rect: &Rect<Au>) {
        // FIXME(#2795): Get the real container size
        let container_size = Size2D::zero();
        // Fragment position wrt to the owning flow.
        let fragment_bounds = self.border_box.to_physical(self.style.writing_mode, container_size);
        let absolute_fragment_bounds = Rect(
            fragment_bounds.origin + flow_origin,
            fragment_bounds.size);

        // Compute the text fragment bounds and draw a border surrounding them.
        let border_display_item = box BorderDisplayItem {
            base: BaseDisplayItem::new(absolute_fragment_bounds,
                                       self.node,
                                       ContentStackingLevel,
                                       *clip_rect),
            border: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::solid)
        };
        display_list.push(BorderDisplayItemClass(border_display_item));

        // Draw a rectangle representing the baselines.
        let ascent = text_fragment.run.ascent();
        let mut baseline = self.border_box.clone();
        baseline.start.b = baseline.start.b + ascent;
        baseline.size.block = Au(0);
        let mut baseline = baseline.to_physical(self.style.writing_mode, container_size);
        baseline.origin = baseline.origin + flow_origin;

        let line_display_item = box LineDisplayItem {
            base: BaseDisplayItem::new(baseline, self.node, ContentStackingLevel, *clip_rect),
            color: rgb(0, 200, 0),
            style: border_style::dashed,
        };
        display_list.push(LineDisplayItemClass(line_display_item));
    }

    fn build_debug_borders_around_fragment(&self,
                                           display_list: &mut DisplayList,
                                           flow_origin: Point2D<Au>,
                                           clip_rect: &Rect<Au>) {
        // FIXME(#2795): Get the real container size
        let container_size = Size2D::zero();
        // Fragment position wrt to the owning flow.
        let fragment_bounds = self.border_box.to_physical(self.style.writing_mode, container_size);
        let absolute_fragment_bounds = Rect(
            fragment_bounds.origin + flow_origin,
            fragment_bounds.size);

        // This prints a debug border around the border of this fragment.
        let border_display_item = box BorderDisplayItem {
            base: BaseDisplayItem::new(absolute_fragment_bounds,
                                       self.node,
                                       ContentStackingLevel,
                                       *clip_rect),
            border: SideOffsets2D::new_all_same(Au::from_px(1)),
            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::solid)
        };
        display_list.push(BorderDisplayItemClass(border_display_item))
    }

    /// Adds the display items for this fragment to the given stacking context.
    ///
    /// Arguments:
    ///
    /// * `display_list`: The unflattened display list to add display items to.
    /// * `layout_context`: The layout context.
    /// * `dirty`: The dirty rectangle in the coordinate system of the owning flow.
    /// * `flow_origin`: Position of the origin of the owning flow wrt the display list root flow.
    /// * `clip_rect`: The rectangle to clip the display items to.
    pub fn build_display_list(&mut self,
                              display_list: &mut DisplayList,
                              layout_context: &LayoutContext,
                              flow_origin: Point2D<Au>,
                              background_and_border_level: BackgroundAndBorderLevel,
                              clip_rect: &Rect<Au>) {
        // FIXME(#2795): Get the real container size
        let container_size = Size2D::zero();
        let rect_to_absolute = |writing_mode: WritingMode, logical_rect: LogicalRect<Au>| {
            let physical_rect = logical_rect.to_physical(writing_mode, container_size);
            Rect(physical_rect.origin + flow_origin, physical_rect.size)
        };
        // Fragment position wrt to the owning flow.
        let absolute_fragment_bounds = rect_to_absolute(self.style.writing_mode, self.border_box);
        debug!("Fragment::build_display_list at rel={}, abs={}: {}",
               self.border_box,
               absolute_fragment_bounds,
               self);
        debug!("Fragment::build_display_list: dirty={}, flow_origin={}",
               layout_context.shared.dirty,
               flow_origin);

        if self.style().get_inheritedbox().visibility != visibility::visible {
            return
        }

        if !absolute_fragment_bounds.intersects(&layout_context.shared.dirty) {
            debug!("Fragment::build_display_list: Did not intersect...");
            return
        }

        debug!("Fragment::build_display_list: intersected. Adding display item...");

        if self.is_primary_fragment() {
            let level =
                StackingLevel::from_background_and_border_level(background_and_border_level);

            // Add a pseudo-display item for content box queries. This is a very bogus thing to do.
            let base_display_item = box BaseDisplayItem::new(absolute_fragment_bounds,
                                                             self.node,
                                                             level,
                                                             *clip_rect);
            display_list.push(PseudoDisplayItemClass(base_display_item));

            // Add the background to the list, if applicable.
            match self.inline_context {
                Some(ref inline_context) => {
                    for style in inline_context.styles.iter().rev() {
                        self.build_display_list_for_background_if_applicable(
                            &**style,
                            display_list,
                            layout_context,
                            level,
                            &absolute_fragment_bounds,
                            clip_rect);
                    }
                }
                None => {}
            }
            match self.specific {
                ScannedTextFragment(_) => {},
                _ => {
                    self.build_display_list_for_background_if_applicable(
                        &*self.style,
                        display_list,
                        layout_context,
                        level,
                        &absolute_fragment_bounds,
                        clip_rect);
                }
            }

            // Add a border, if applicable.
            //
            // TODO: Outlines.
            match self.inline_context {
                Some(ref inline_context) => {
                    for style in inline_context.styles.iter().rev() {
                        self.build_display_list_for_borders_if_applicable(
                            &**style,
                            display_list,
                            &absolute_fragment_bounds,
                            level,
                            clip_rect);
                    }
                }
                None => {}
            }
            match self.specific {
                ScannedTextFragment(_) => {},
                _ => {
                    self.build_display_list_for_borders_if_applicable(
                        &*self.style,
                        display_list,
                        &absolute_fragment_bounds,
                        level,
                        clip_rect);
                }
            }
        }

        let content_box = self.content_box();
        let absolute_content_box = rect_to_absolute(self.style.writing_mode, content_box);

        // Create special per-fragment-type display items.
        match self.specific {
            UnscannedTextFragment(_) => fail!("Shouldn't see unscanned fragments here."),
            TableColumnFragment(_) => fail!("Shouldn't see table column fragments here."),
            ScannedTextFragment(ref text_fragment) => {
                // Create the text display item.
                let orientation = if self.style.writing_mode.is_vertical() {
                    if self.style.writing_mode.is_sideways_left() {
                        SidewaysLeft
                    } else {
                        SidewaysRight
                    }
                } else {
                    Upright
                };

                let metrics = &text_fragment.run.font_metrics;
                let baseline_origin ={
                    let mut tmp = content_box.start;
                    tmp.b = tmp.b + metrics.ascent;
                    tmp.to_physical(self.style.writing_mode, container_size) + flow_origin
                };

                let text_display_item = box TextDisplayItem {
                    base: BaseDisplayItem::new(absolute_content_box,
                                               self.node,
                                               ContentStackingLevel,
                                               *clip_rect),
                    text_run: text_fragment.run.clone(),
                    range: text_fragment.range,
                    text_color: self.style().get_color().color.to_gfx_color(),
                    orientation: orientation,
                    baseline_origin: baseline_origin,
                };
                display_list.push(TextDisplayItemClass(text_display_item));

                // Create display items for text decoration
                {
                    let line = |maybe_color: Option<RGBA>, rect: || -> LogicalRect<Au>| {
                        match maybe_color {
                            None => {},
                            Some(color) => {
                                display_list.push(SolidColorDisplayItemClass(
                                                     box SolidColorDisplayItem {
                                                        base: BaseDisplayItem::new(
                                                                  rect_to_absolute(
                                                                      self.style.writing_mode,
                                                                      rect()),
                                                               self.node,
                                                               ContentStackingLevel,
                                                               *clip_rect),
                                                        color: color.to_gfx_color(),
                                                     }));
                            }
                        }
                    };

                    let text_decorations =
                        self.style().get_inheritedtext()._servo_text_decorations_in_effect;
                    line(text_decorations.underline, || {
                        let mut rect = content_box.clone();
                        rect.start.b = rect.start.b + metrics.ascent - metrics.underline_offset;
                        rect.size.block = metrics.underline_size;
                        rect
                    });

                    line(text_decorations.overline, || {
                        let mut rect = content_box.clone();
                        rect.size.block = metrics.underline_size;
                        rect
                    });

                    line(text_decorations.line_through, || {
                        let mut rect = content_box.clone();
                        rect.start.b = rect.start.b + metrics.ascent - metrics.strikeout_offset;
                        rect.size.block = metrics.strikeout_size;
                        rect
                    });
                }

                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_text_fragments(display_list,
                                                                   flow_origin,
                                                                   text_fragment,
                                                                   clip_rect);
                }
            }
            GenericFragment | IframeFragment(..) | TableFragment | TableCellFragment |
            TableRowFragment | TableWrapperFragment | InlineBlockFragment(_) | InputFragment |
            InlineAbsoluteHypotheticalFragment(_) => {
                if opts::get().show_debug_fragment_borders {
                    self.build_debug_borders_around_fragment(display_list,
                                                             flow_origin,
                                                             clip_rect);
                }
            }
            ImageFragment(ref mut image_fragment) => {
                let image_ref = &mut image_fragment.image;
                match image_ref.get_image(self.node.to_untrusted_node_address()) {
                    Some(image) => {
                        debug!("(building display list) building image fragment");

                        // Place the image into the display list.
                        let image_display_item = box ImageDisplayItem {
                            base: BaseDisplayItem::new(absolute_content_box,
                                                       self.node,
                                                       ContentStackingLevel,
                                                       *clip_rect),
                            image: image.clone(),
                            stretch_size: absolute_content_box.size,
                        };

                        display_list.push(ImageDisplayItemClass(image_display_item))
                    }
                    None => {
                        // No image data at all? Do nothing.
                        //
                        // TODO: Add some kind of placeholder image.
                        debug!("(building display list) no image :(");
                    }
                }
            }
        }

        // FIXME(pcwalton): This is a bit of an abuse of the logging
        // infrastructure. We should have a real `SERVO_DEBUG` system.
        debug!("{:?}",
               self.build_debug_borders_around_fragment(display_list, flow_origin, clip_rect))

        // If this is an iframe, then send its position and size up to the constellation.
        //
        // FIXME(pcwalton): Doing this during display list construction seems potentially
        // problematic if iframes are outside the area we're computing the display list for, since
        // they won't be able to reflow at all until the user scrolls to them. Perhaps we should
        // separate this into two parts: first we should send the size only to the constellation
        // once that's computed during assign-block-sizes, and second we should should send the
        // origin to the constellation here during display list construction. This should work
        // because layout for the iframe only needs to know size, and origin is only relevant if
        // the iframe is actually going to be displayed.
        match self.specific {
            IframeFragment(ref iframe_fragment) => {
                self.finalize_position_and_size_of_iframe(iframe_fragment,
                                                          absolute_fragment_bounds.origin,
                                                          layout_context)
            }
            _ => {}
        }
    }

    /// Computes the intrinsic inline-sizes of this fragment.
    pub fn compute_intrinsic_inline_sizes(&mut self) -> IntrinsicISizesContribution {
        let mut result = self.style_specified_intrinsic_inline_size();
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment |
            TableColumnFragment(_) | TableRowFragment | TableWrapperFragment |
            InlineAbsoluteHypotheticalFragment(_) | InputFragment => {}
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
                fail!("Unscanned text fragments should have been scanned by now!")
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
            InputFragment | InlineAbsoluteHypotheticalFragment(_) => Au(0),
            ImageFragment(ref image_fragment_info) => {
                image_fragment_info.computed_inline_size()
            }
            ScannedTextFragment(ref text_fragment_info) => {
                let (range, run) = (&text_fragment_info.range, &text_fragment_info.run);
                let text_bounds = run.metrics_for_range(range).bounding_box;
                text_bounds.size.width
            }
            TableColumnFragment(_) => fail!("Table column fragments do not have inline_size"),
            UnscannedTextFragment(_) => fail!("Unscanned text fragments should have been scanned by now!"),
        }
    }

    /// Returns, and computes, the block-size of this fragment.
    pub fn content_block_size(&self, layout_context: &LayoutContext) -> Au {
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment |
            TableRowFragment | TableWrapperFragment | InlineBlockFragment(_) |
            InputFragment | InlineAbsoluteHypotheticalFragment(_) => Au(0),
            ImageFragment(ref image_fragment_info) => {
                image_fragment_info.computed_block_size()
            }
            ScannedTextFragment(_) => {
                // Compute the block-size based on the line-block-size and font size.
                self.calculate_line_height(layout_context)
            }
            TableColumnFragment(_) => fail!("Table column fragments do not have block_size"),
            UnscannedTextFragment(_) => fail!("Unscanned text fragments should have been scanned by now!"),
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
            TableRowFragment | TableWrapperFragment | InputFragment => None,
            TableColumnFragment(_) => fail!("Table column fragments do not need to split"),
            UnscannedTextFragment(_) => fail!("Unscanned text fragments should have been scanned by now!"),
            InlineBlockFragment(_) | InlineAbsoluteHypotheticalFragment(_) => {
                fail!("Inline blocks or inline absolute hypothetical fragments do not get split")
            }
            ScannedTextFragment(ref text_fragment_info) => {
                let mut new_line_pos = self.new_line_pos.clone();
                let cur_new_line_pos = new_line_pos.remove(0).unwrap();

                let inline_start_range = Range::new(text_fragment_info.range.begin(), cur_new_line_pos);
                let inline_end_range = Range::new(text_fragment_info.range.begin() + cur_new_line_pos + CharIndex(1),
                                             text_fragment_info.range.length() - (cur_new_line_pos + CharIndex(1)));

                // Left fragment is for inline-start text of first founded new-line character.
                let inline_start_fragment = SplitInfo::new(inline_start_range, text_fragment_info);

                // Right fragment is for inline-end text of first founded new-line character.
                let inline_end_fragment = if inline_end_range.length() > CharIndex(0) {
                    Some(SplitInfo::new(inline_end_range, text_fragment_info))
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
    //
    // TODO(bjz): The text run should be removed in the future, but it is currently needed for
    // the current method of fragment splitting in the `inline::try_append_*` functions.
    pub fn find_split_info_for_inline_size(&self, start: CharIndex, max_inline_size: Au, starts_line: bool)
            -> Option<(Option<SplitInfo>, Option<SplitInfo>, Arc<Box<TextRun>> /* TODO(bjz): remove */)> {
        match self.specific {
            GenericFragment | IframeFragment(_) | ImageFragment(_) | TableFragment | TableCellFragment |
            TableRowFragment | TableWrapperFragment | InlineBlockFragment(_) | InputFragment |
            InlineAbsoluteHypotheticalFragment(_) => None,
            TableColumnFragment(_) => fail!("Table column fragments do not have inline_size"),
            UnscannedTextFragment(_) => fail!("Unscanned text fragments should have been scanned by now!"),
            ScannedTextFragment(ref text_fragment_info) => {
                let mut pieces_processed_count: uint = 0;
                let mut remaining_inline_size: Au = max_inline_size;
                let mut inline_start_range = Range::new(text_fragment_info.range.begin() + start, CharIndex(0));
                let mut inline_end_range: Option<Range<CharIndex>> = None;

                debug!("split_to_inline_size: splitting text fragment (strlen={}, range={}, avail_inline_size={})",
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
                            debug!("split_to_inline_size: case=splitting remainder with inline_end range={:?}",
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
                        Some(SplitInfo::new(inline_start_range, text_fragment_info))
                    } else {
                         None
                    };
                    let inline_end = inline_end_range.map(|inline_end_range| {
                        SplitInfo::new(inline_end_range, text_fragment_info)
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
            TableRowFragment | TableWrapperFragment | InputFragment => return,
            TableColumnFragment(_) => fail!("Table column fragments do not have inline_size"),
            UnscannedTextFragment(_) => {
                fail!("Unscanned text fragments should have been scanned by now!")
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
                self.border_box.size.inline = info.content_inline_size + noncontent_inline_size
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
            _ => fail!("this case should have been handled above"),
        }
    }

    /// Assign block-size for this fragment if it is replaced content. The inline-size must have
    /// been assigned first.
    ///
    /// Ideally, this should follow CSS 2.1 § 10.6.2.
    pub fn assign_replaced_block_size_if_necessary(&mut self, containing_block_block_size: Au) {
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment |
            TableRowFragment | TableWrapperFragment | InputFragment => return,
            TableColumnFragment(_) => fail!("Table column fragments do not have block_size"),
            UnscannedTextFragment(_) => {
                fail!("Unscanned text fragments should have been scanned by now!")
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
            ScannedTextFragment(_) => {
                // Scanned text fragments' content block-sizes are calculated by the text run
                // scanner during flow construction.
                self.border_box.size.block = self.border_box.size.block + noncontent_block_size
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
            _ => fail!("should have been handled above"),
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
                let font_style = self.style.get_font();
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

    /// Sends the size and position of this iframe fragment to the constellation. This is out of
    /// line to guide inlining.
    #[inline(never)]
    fn finalize_position_and_size_of_iframe(&self,
                                            iframe_fragment: &IframeFragmentInfo,
                                            offset: Point2D<Au>,
                                            layout_context: &LayoutContext) {
        let border_padding = (self.border_padding).to_physical(self.style.writing_mode);
        let content_size = self.content_box().size.to_physical(self.style.writing_mode);
        let iframe_rect = Rect(Point2D(geometry::to_frac_px(offset.x + border_padding.left) as f32,
                                       geometry::to_frac_px(offset.y + border_padding.top) as f32),
                               Size2D(geometry::to_frac_px(content_size.width) as f32,
                                      geometry::to_frac_px(content_size.height) as f32));

        debug!("finalizing position and size of iframe for {:?},{:?}",
               iframe_fragment.pipeline_id,
               iframe_fragment.subpage_id);
        let ConstellationChan(ref chan) = layout_context.shared.constellation_chan;
        chan.send(FrameRectMsg(iframe_fragment.pipeline_id,
                               iframe_fragment.subpage_id,
                               iframe_rect));
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
    fn is_primary_fragment(&self) -> bool {
        match self.specific {
            InlineBlockFragment(_) | InlineAbsoluteHypotheticalFragment(_) |
            TableWrapperFragment => false,
            GenericFragment | IframeFragment(_) | ImageFragment(_) | ScannedTextFragment(_) |
            TableFragment | TableCellFragment | TableColumnFragment(_) | TableRowFragment |
            UnscannedTextFragment(_) | InputFragment => true,
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

    pub fn clip_rect_for_children(&self, current_clip_rect: Rect<Au>, flow_origin: Point2D<Au>)
                                  -> Rect<Au> {
        // Don't clip if we're text.
        match self.specific {
            ScannedTextFragment(_) => return current_clip_rect,
            _ => {}
        }

        // Only clip if `overflow` tells us to.
        match self.style.get_box().overflow {
            overflow::hidden | overflow::auto | overflow::scroll => {}
            _ => return current_clip_rect,
        }

        // Create a new clip rect.
        //
        // FIXME(#2795): Get the real container size.
        let physical_rect = self.border_box.to_physical(self.style.writing_mode, Size2D::zero());
        current_clip_rect.intersection(&Rect(physical_rect.origin + flow_origin,
                                             physical_rect.size)).unwrap_or(ZERO_RECT)
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
        static IntrinsicInlineSizeIncludesMargins = 0x01,
        static IntrinsicInlineSizeIncludesPadding = 0x02,
        static IntrinsicInlineSizeIncludesBorder = 0x04,
        static IntrinsicInlineSizeIncludesSpecified = 0x08,
    }
}
