/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Fragment` type, which represents the leaves of the layout tree.

#![deny(unsafe_code)]

use app_units::Au;
use canvas_traits::CanvasMsg;
use context::LayoutContext;
use euclid::{Point2D, Rect, Size2D};
use floats::ClearType;
use flow::{self, ImmutableFlowUtils};
use flow_ref::{self, FlowRef};
use gfx;
use gfx::display_list::{BLUR_INFLATION_FACTOR, OpaqueNode};
use gfx::text::glyph::ByteIndex;
use gfx::text::text_run::{TextRun, TextRunSlice};
use gfx_traits::{FragmentType, LayerId, LayerType, StackingContextId};
use inline::{FIRST_FRAGMENT_OF_ELEMENT, InlineFragmentContext, InlineFragmentNodeInfo};
use inline::{InlineMetrics, LAST_FRAGMENT_OF_ELEMENT};
use ipc_channel::ipc::IpcSender;
#[cfg(debug_assertions)]
use layout_debug;
use model::{self, IntrinsicISizes, IntrinsicISizesContribution, MaybeAuto, specified};
use msg::constellation_msg::PipelineId;
use net_traits::image::base::{Image, ImageMetadata};
use net_traits::image_cache_thread::{ImageOrMetadataAvailable, UsePlaceholder};
use range::*;
use rustc_serialize::{Encodable, Encoder};
use script_layout_interface::HTMLCanvasData;
use script_layout_interface::restyle_damage::{RECONSTRUCT_FLOW, RestyleDamage};
use script_layout_interface::wrapper_traits::{PseudoElementType, ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use std::borrow::ToOwned;
use std::cmp::{max, min};
use std::collections::LinkedList;
use std::fmt;
use std::sync::{Arc, Mutex};
use style::arc_ptr_eq;
use style::computed_values::content::ContentItem;
use style::computed_values::{border_collapse, clear, color, display, mix_blend_mode};
use style::computed_values::{overflow_wrap, overflow_x, position, text_decoration};
use style::computed_values::{transform_style, vertical_align, white_space, word_break, z_index};
use style::dom::TRestyleDamage;
use style::logical_geometry::{LogicalMargin, LogicalRect, LogicalSize, WritingMode};
use style::properties::ServoComputedValues;
use style::str::char_is_whitespace;
use style::values::computed::LengthOrPercentageOrNone;
use style::values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto};
use text;
use text::TextRunScanner;
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
    pub style: Arc<ServoComputedValues>,

    /// The CSS style of this fragment when it's selected
    pub selected_style: Arc<ServoComputedValues>,

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

impl Encodable for Fragment {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_struct("fragment", 0, |e| {
            try!(e.emit_struct_field("id", 0, |e| self.debug_id.encode(e)));
            try!(e.emit_struct_field("border_box", 1, |e| self.border_box.encode(e)));
            e.emit_struct_field("margin", 2, |e| self.margin.encode(e))
        })
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
                SpecificFragmentInfo::Table |
                SpecificFragmentInfo::TableCell |
                SpecificFragmentInfo::TableColumn(_) |
                SpecificFragmentInfo::TableRow |
                SpecificFragmentInfo::TableWrapper |
                SpecificFragmentInfo::Multicol |
                SpecificFragmentInfo::MulticolColumn |
                SpecificFragmentInfo::UnscannedText(_) |
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
            SpecificFragmentInfo::Table => "SpecificFragmentInfo::Table",
            SpecificFragmentInfo::TableCell => "SpecificFragmentInfo::TableCell",
            SpecificFragmentInfo::TableColumn(_) => "SpecificFragmentInfo::TableColumn",
            SpecificFragmentInfo::TableRow => "SpecificFragmentInfo::TableRow",
            SpecificFragmentInfo::TableWrapper => "SpecificFragmentInfo::TableWrapper",
            SpecificFragmentInfo::Multicol => "SpecificFragmentInfo::Multicol",
            SpecificFragmentInfo::MulticolColumn => "SpecificFragmentInfo::MulticolColumn",
            SpecificFragmentInfo::UnscannedText(_) => "SpecificFragmentInfo::UnscannedText",
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

/// Clamp a value obtained from style_length, based on min / max lengths.
fn clamp_size(size: Au,
              min_size: LengthOrPercentage,
              max_size: LengthOrPercentageOrNone,
              container_size: Au)
              -> Au {
    let min_size = model::specified(min_size, container_size);
    let max_size = model::specified_or_none(max_size, container_size);

    max(min_size, match max_size {
        None => size,
        Some(max_size) => min(size, max_size),
    })
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
    pub replaced_image_fragment_info: ReplacedImageFragmentInfo,
    pub ipc_renderer: Option<Arc<Mutex<IpcSender<CanvasMsg>>>>,
    pub dom_width: Au,
    pub dom_height: Au,
}

impl CanvasFragmentInfo {
    pub fn new<N: ThreadSafeLayoutNode>(node: &N, data: HTMLCanvasData, ctx: &LayoutContext) -> CanvasFragmentInfo {
        CanvasFragmentInfo {
            replaced_image_fragment_info: ReplacedImageFragmentInfo::new(node, ctx),
            ipc_renderer: data.ipc_renderer
                              .map(|renderer| Arc::new(Mutex::new(renderer))),
            dom_width: Au::from_px(data.width as i32),
            dom_height: Au::from_px(data.height as i32),
        }
    }

    /// Returns the original inline-size of the canvas.
    pub fn canvas_inline_size(&self) -> Au {
        if self.replaced_image_fragment_info.writing_mode_is_vertical {
            self.dom_height
        } else {
            self.dom_width
        }
    }

    /// Returns the original block-size of the canvas.
    pub fn canvas_block_size(&self) -> Au {
        if self.replaced_image_fragment_info.writing_mode_is_vertical {
            self.dom_width
        } else {
            self.dom_height
        }
    }
}


/// A fragment that represents a replaced content image and its accompanying borders, shadows, etc.
#[derive(Clone)]
pub struct ImageFragmentInfo {
    /// The image held within this fragment.
    pub replaced_image_fragment_info: ReplacedImageFragmentInfo,
    pub image: Option<Arc<Image>>,
    pub metadata: Option<ImageMetadata>,
}

impl ImageFragmentInfo {
    /// Creates a new image fragment from the given URL and local image cache.
    ///
    /// FIXME(pcwalton): The fact that image fragments store the cache in the fragment makes little
    /// sense to me.
    pub fn new<N: ThreadSafeLayoutNode>(node: &N, url: Option<Url>,
                                        layout_context: &LayoutContext) -> ImageFragmentInfo {
        let image_or_metadata = url.and_then(|url| {
            layout_context.get_or_request_image_or_meta(url, UsePlaceholder::Yes)
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
            replaced_image_fragment_info: ReplacedImageFragmentInfo::new(node, layout_context),
            image: image,
            metadata: metadata,
        }
    }

    /// Returns the original inline-size of the image.
    pub fn image_inline_size(&mut self) -> Au {
        match self.metadata {
            Some(ref metadata) => {
                Au::from_px(if self.replaced_image_fragment_info.writing_mode_is_vertical {
                    metadata.height
                } else {
                    metadata.width
                } as i32)
            }
            None => Au(0)
        }
    }

    /// Returns the original block-size of the image.
    pub fn image_block_size(&mut self) -> Au {
        match self.metadata {
            Some(ref metadata) => {
                Au::from_px(if self.replaced_image_fragment_info.writing_mode_is_vertical {
                    metadata.width
                } else {
                    metadata.height
                } as i32)
            }
            None => Au(0)
        }
    }

    /// Tile an image
    pub fn tile_image(position: &mut Au, size: &mut Au, virtual_position: Au, image_size: u32) {
        // Avoid division by zero below!
        let image_size = image_size as i32;
        if image_size == 0 {
            return
        }

        let delta_pixels = (virtual_position - *position).to_px();
        let tile_count = (delta_pixels + image_size - 1) / image_size;
        let offset = Au::from_px(image_size * tile_count);
        let new_position = virtual_position - offset;
        *size = *position - new_position + *size;
        *position = new_position;
    }
}

#[derive(Clone)]
pub struct ReplacedImageFragmentInfo {
    pub computed_inline_size: Option<Au>,
    pub computed_block_size: Option<Au>,
    pub writing_mode_is_vertical: bool,
}

impl ReplacedImageFragmentInfo {
    pub fn new<N>(node: &N, ctx: &LayoutContext) -> ReplacedImageFragmentInfo
            where N: ThreadSafeLayoutNode {
        let is_vertical = node.style(ctx.style_context()).writing_mode.is_vertical();
        ReplacedImageFragmentInfo {
            computed_inline_size: None,
            computed_block_size: None,
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

    // Return used value for inline-size or block-size.
    //
    // `dom_length`: inline-size or block-size as specified in the `img` tag.
    // `style_length`: inline-size as given in the CSS
    pub fn style_length(style_length: LengthOrPercentageOrAuto,
                        container_size: Option<Au>) -> MaybeAuto {
        match (style_length, container_size) {
            (LengthOrPercentageOrAuto::Length(length), _) => MaybeAuto::Specified(length),
            (LengthOrPercentageOrAuto::Percentage(pc), Some(container_size)) => {
                MaybeAuto::Specified(container_size.scale_by(pc))
            }
            (LengthOrPercentageOrAuto::Percentage(_), None) => MaybeAuto::Auto,
            (LengthOrPercentageOrAuto::Calc(calc), Some(container_size)) => {
                MaybeAuto::Specified(calc.length() + container_size.scale_by(calc.percentage()))
            }
            (LengthOrPercentageOrAuto::Calc(_), None) => MaybeAuto::Auto,
            (LengthOrPercentageOrAuto::Auto, _) => MaybeAuto::Auto,
        }
    }

    pub fn calculate_replaced_inline_size(&mut self,
                                          style: &ServoComputedValues,
                                          noncontent_inline_size: Au,
                                          container_inline_size: Au,
                                          container_block_size: Option<Au>,
                                          fragment_inline_size: Au,
                                          fragment_block_size: Au)
                                          -> Au {
        let style_inline_size = style.content_inline_size();
        let style_block_size = style.content_block_size();
        let style_min_inline_size = style.min_inline_size();
        let style_max_inline_size = style.max_inline_size();
        let style_min_block_size = style.min_block_size();
        let style_max_block_size = style.max_block_size();

        // TODO(ksh8281): compute border,margin
        let inline_size = ReplacedImageFragmentInfo::style_length(
            style_inline_size,
            Some(container_inline_size));

        let inline_size = match inline_size {
            MaybeAuto::Auto => {
                let intrinsic_width = fragment_inline_size;
                let intrinsic_height = fragment_block_size;
                if intrinsic_height == Au(0) {
                    intrinsic_width
                } else {
                    let ratio = intrinsic_width.to_f32_px() /
                                intrinsic_height.to_f32_px();

                    let specified_height = ReplacedImageFragmentInfo::style_length(
                        style_block_size,
                        container_block_size);
                    let specified_height = match specified_height {
                        MaybeAuto::Auto => intrinsic_height,
                        MaybeAuto::Specified(h) => h,
                    };
                    let specified_height = clamp_size(specified_height,
                                                      style_min_block_size,
                                                      style_max_block_size,
                                                      Au(0));
                    Au::from_f32_px(specified_height.to_f32_px() * ratio)
                }
            },
            MaybeAuto::Specified(w) => w,
        };

        let inline_size = clamp_size(inline_size,
                                     style_min_inline_size,
                                     style_max_inline_size,
                                     container_inline_size);

        self.computed_inline_size = Some(inline_size);
        inline_size + noncontent_inline_size
    }

    pub fn calculate_replaced_block_size(&mut self,
                                         style: &ServoComputedValues,
                                         noncontent_block_size: Au,
                                         containing_block_block_size: Option<Au>,
                                         fragment_inline_size: Au,
                                         fragment_block_size: Au)
                                         -> Au {
        // TODO(ksh8281): compute border,margin,padding
        let style_block_size = style.content_block_size();
        let style_min_block_size = style.min_block_size();
        let style_max_block_size = style.max_block_size();

        let inline_size = self.computed_inline_size();
        let block_size = ReplacedImageFragmentInfo::style_length(
            style_block_size,
            containing_block_block_size);

        let block_size = match block_size {
            MaybeAuto::Auto => {
                let intrinsic_width = fragment_inline_size;
                let intrinsic_height = fragment_block_size;
                let scale = intrinsic_width.to_f32_px() / inline_size.to_f32_px();
                Au::from_f32_px(intrinsic_height.to_f32_px() / scale)
            },
            MaybeAuto::Specified(h) => {
                h
            }
        };

        let block_size = clamp_size(block_size,
                                    style_min_block_size,
                                    style_max_block_size,
                                    Au(0));

        self.computed_block_size = Some(block_size);
        block_size + noncontent_block_size
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

    #[inline]
    pub fn calculate_replaced_inline_size(&self, style: &ServoComputedValues, containing_size: Au)
                                          -> Au {
        // Calculate the replaced inline size (or default) as per CSS 2.1 § 10.3.2
        IframeFragmentInfo::calculate_replaced_size(style.content_inline_size(),
                                                    style.min_inline_size(),
                                                    style.max_inline_size(),
                                                    Some(containing_size),
                                                    Au::from_px(300))
    }

    #[inline]
    pub fn calculate_replaced_block_size(&self, style: &ServoComputedValues, containing_size: Option<Au>)
                                         -> Au {
        // Calculate the replaced block size (or default) as per CSS 2.1 § 10.3.2
        IframeFragmentInfo::calculate_replaced_size(style.content_block_size(),
                                                    style.min_block_size(),
                                                    style.max_block_size(),
                                                    containing_size,
                                                    Au::from_px(150))

    }

    fn calculate_replaced_size(content_size: LengthOrPercentageOrAuto,
                               style_min_size: LengthOrPercentage,
                               style_max_size: LengthOrPercentageOrNone,
                               containing_size: Option<Au>,
                               default_size: Au) -> Au {
        let computed_size = match (content_size, containing_size) {
            (LengthOrPercentageOrAuto::Length(length), _) => length,
            (LengthOrPercentageOrAuto::Percentage(pc), Some(container_size)) => container_size.scale_by(pc),
            (LengthOrPercentageOrAuto::Calc(calc), Some(container_size)) => {
                container_size.scale_by(calc.percentage()) + calc.length()
            },
            (LengthOrPercentageOrAuto::Calc(calc), None) => calc.length(),
            (LengthOrPercentageOrAuto::Percentage(_), None) => default_size,
            (LengthOrPercentageOrAuto::Auto, _) => default_size,
        };

        let containing_size = containing_size.unwrap_or(Au(0));
        clamp_size(computed_size,
                   style_min_size,
                   style_max_size,
                   containing_size)
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
pub struct TruncationResult {
    /// The part of the fragment remaining after truncation.
    pub split: SplitInfo,
    /// The text run which is being truncated.
    pub text_run: Arc<TextRun>,
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
        let element = node.as_element();
        let span = element.get_attr(&ns!(), &atom!("span"))
                          .and_then(|string| string.parse().ok())
                          .unwrap_or(0);
        TableColumnFragmentInfo {
            span: span,
        }
    }
}

impl Fragment {
    /// Constructs a new `Fragment` instance.
    pub fn new<N: ThreadSafeLayoutNode>(node: &N, specific: SpecificFragmentInfo, ctx: &LayoutContext) -> Fragment {
        let style_context = ctx.style_context();
        let style = node.style(style_context).clone();
        let writing_mode = style.writing_mode;

        let mut restyle_damage = node.restyle_damage();
        restyle_damage.remove(RECONSTRUCT_FLOW);

        Fragment {
            node: node.opaque(),
            style: style,
            selected_style: node.selected_style(style_context).clone(),
            restyle_damage: restyle_damage,
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: specific,
            inline_context: None,
            pseudo: node.get_pseudo_element_type().strip(),
            flags: FragmentFlags::empty(),
            debug_id: DebugId::new(),
            stacking_context_id: StackingContextId::new(0),
        }
    }

    /// Constructs a new `Fragment` instance from an opaque node.
    pub fn from_opaque_node_and_style(node: OpaqueNode,
                                      pseudo: PseudoElementType<()>,
                                      style: Arc<ServoComputedValues>,
                                      selected_style: Arc<ServoComputedValues>,
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
            stacking_context_id: StackingContextId::new(0),
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
            stacking_context_id: StackingContextId::new(0),
        }
    }

    /// Transforms this fragment using the given `SplitInfo`, preserving all the other data.
    pub fn transform_with_split_info(&self, split: &SplitInfo, text_run: Arc<TextRun>)
                                     -> Fragment {
        let size = LogicalSize::new(self.style.writing_mode,
                                    split.inline_size,
                                    self.border_box.size.block);
        let flags = match self.specific {
            SpecificFragmentInfo::ScannedText(ref info) => info.flags,
            _ => ScannedTextFlags::empty()
        };
        // FIXME(pcwalton): This should modify the insertion point as necessary.
        let info = box ScannedTextFragmentInfo::new(
            text_run,
            split.range,
            size,
            None,
            flags);
        self.transform(size, SpecificFragmentInfo::ScannedText(info))
    }

    /// Transforms this fragment into an ellipsis fragment, preserving all the other data.
    pub fn transform_into_ellipsis(&self, layout_context: &LayoutContext) -> Fragment {
        let mut unscanned_ellipsis_fragments = LinkedList::new();
        unscanned_ellipsis_fragments.push_back(self.transform(
                self.border_box.size,
                SpecificFragmentInfo::UnscannedText(
                    box UnscannedTextFragmentInfo::new("…".to_owned(), None))));
        let ellipsis_fragments = TextRunScanner::new().scan_for_runs(&mut layout_context.font_context(),
                                                                     unscanned_ellipsis_fragments);
        debug_assert!(ellipsis_fragments.len() == 1);
        ellipsis_fragments.fragments.into_iter().next().unwrap()
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
            SpecificFragmentInfo::Multicol => {
                QuantitiesIncludedInIntrinsicInlineSizes::all()
            }
            SpecificFragmentInfo::Table | SpecificFragmentInfo::TableCell => {
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
        let mut specified = Au(0);

        if flags.contains(INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED) {
            specified = MaybeAuto::from_style(style.content_inline_size(),
                                              Au(0)).specified_or_zero();
            specified = max(model::specified(style.min_inline_size(), Au(0)), specified);
            if let Some(max) = model::specified_or_none(style.max_inline_size(), Au(0)) {
                specified = min(specified, max)
            }
        }

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

    pub fn calculate_line_height(&self, layout_context: &LayoutContext) -> Au {
        let font_style = self.style.get_font_arc();
        let font_metrics = text::font_metrics_for_style(&mut layout_context.font_context(),
                                                        font_style);
        text::line_height_from_style(&*self.style, &font_metrics)
    }

    /// Returns the sum of the inline-sizes of all the borders of this fragment. Note that this
    /// can be expensive to compute, so if possible use the `border_padding` field instead.
    #[inline]
    pub fn border_width(&self) -> LogicalMargin<Au> {
        let style_border_width = match self.specific {
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::InlineBlock(_) => LogicalMargin::zero(self.style.writing_mode),
            _ => self.style().logical_border_width(),
        };

        match self.inline_context {
            None => style_border_width,
            Some(ref inline_fragment_context) => {
                inline_fragment_context.nodes.iter().fold(style_border_width, |accumulator, node| {
                    let mut this_border_width = node.style.logical_border_width();
                    if !node.flags.contains(FIRST_FRAGMENT_OF_ELEMENT) {
                        this_border_width.inline_start = Au(0)
                    }
                    if !node.flags.contains(LAST_FRAGMENT_OF_ELEMENT) {
                        this_border_width.inline_end = Au(0)
                    }
                    accumulator + this_border_width
                })
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
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableColumn(_) => {
                self.margin.inline_start = Au(0);
                self.margin.inline_end = Au(0);
                return
            }
            SpecificFragmentInfo::InlineBlock(_) => {
                // Inline-blocks do not take self margins into account but do account for margins
                // from outer inline contexts.
                self.margin.inline_start = Au(0);
                self.margin.inline_end = Au(0);
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

                self.margin.inline_start = self.margin.inline_start + this_inline_start_margin;
                self.margin.inline_end = self.margin.inline_end + this_inline_end_margin;
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
        //
        // This is zero in the case of `inline-block` because that padding is applied to the
        // wrapped block, not the fragment.
        let padding_from_style = match self.specific {
            SpecificFragmentInfo::TableColumn(_) |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableWrapper |
            SpecificFragmentInfo::InlineBlock(_) => LogicalMargin::zero(self.style.writing_mode),
            _ => model::padding_from_style(self.style(), containing_block_inline_size),
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
                let zero_padding = LogicalMargin::zero(self.style.writing_mode);
                inline_fragment_context.nodes.iter().fold(zero_padding, |accumulator, node| {
                    let mut padding = model::padding_from_style(&*node.style, Au(0));
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
                                      container_size.inline).specified_or_zero()
            } else {
                -MaybeAuto::from_style(offsets.block_end,
                                       container_size.inline).specified_or_zero()
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
            SpecificFragmentInfo::Iframe(_) |
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
            SpecificFragmentInfo::Image(ref mut image_fragment_info) => {
                let mut image_inline_size = match self.style.content_inline_size() {
                    LengthOrPercentageOrAuto::Auto |
                    LengthOrPercentageOrAuto::Percentage(_) => {
                        image_fragment_info.image_inline_size()
                    }
                    LengthOrPercentageOrAuto::Length(length) => length,
                    LengthOrPercentageOrAuto::Calc(calc) => calc.length(),
                };

                image_inline_size = max(model::specified(self.style.min_inline_size(), Au(0)), image_inline_size);
                if let Some(max) = model::specified_or_none(self.style.max_inline_size(), Au(0)) {
                    image_inline_size = min(image_inline_size, max)
                }

                result.union_block(&IntrinsicISizes {
                    minimum_inline_size: image_inline_size,
                    preferred_inline_size: image_inline_size,
                });
            }
            SpecificFragmentInfo::Canvas(ref mut canvas_fragment_info) => {
                let mut canvas_inline_size = match self.style.content_inline_size() {
                    LengthOrPercentageOrAuto::Auto |
                    LengthOrPercentageOrAuto::Percentage(_) => {
                        canvas_fragment_info.canvas_inline_size()
                    }
                    LengthOrPercentageOrAuto::Length(length) => length,
                    LengthOrPercentageOrAuto::Calc(calc) => calc.length(),
                };

                canvas_inline_size = max(model::specified(self.style.min_inline_size(), Au(0)), canvas_inline_size);
                if let Some(max) = model::specified_or_none(self.style.max_inline_size(), Au(0)) {
                    canvas_inline_size = min(canvas_inline_size, max)
                }

                result.union_block(&IntrinsicISizes {
                    minimum_inline_size: canvas_inline_size,
                    preferred_inline_size: canvas_inline_size,
                });
            }
            SpecificFragmentInfo::ScannedText(ref text_fragment_info) => {
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
            SpecificFragmentInfo::UnscannedText(..) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            }
        };

        // Take borders and padding for parent inline fragments into account, if necessary.
        if self.is_primary_fragment() {
            if let Some(ref context) = self.inline_context {
                for node in &context.nodes {
                    let mut border_width = node.style.logical_border_width();
                    let mut padding = model::padding_from_style(&*node.style, Au(0));
                    let mut margin = model::specified_margin_from_style(&*node.style);
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
        }

        result
    }

    /// Returns the narrowest inline-size that the first splittable part of this fragment could
    /// possibly be split to. (In most cases, this returns the inline-size of the first word in
    /// this fragment.)
    pub fn minimum_splittable_inline_size(&self) -> Au {
        match self.specific {
            SpecificFragmentInfo::ScannedText(ref text) => {
                text.run.minimum_splittable_inline_size(&text.range)
            }
            _ => Au(0),
        }
    }

    /// TODO: What exactly does this function return? Why is it Au(0) for
    /// `SpecificFragmentInfo::Generic`?
    pub fn content_inline_size(&self) -> Au {
        match self.specific {
            SpecificFragmentInfo::Generic |
            SpecificFragmentInfo::GeneratedContent(_) |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::TableWrapper |
            SpecificFragmentInfo::Multicol |
            SpecificFragmentInfo::MulticolColumn |
            SpecificFragmentInfo::InlineBlock(_) |
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) |
            SpecificFragmentInfo::InlineAbsolute(_) => Au(0),
            SpecificFragmentInfo::Canvas(ref canvas_fragment_info) => {
                canvas_fragment_info.replaced_image_fragment_info.computed_inline_size()
            }
            SpecificFragmentInfo::Image(ref image_fragment_info) => {
                image_fragment_info.replaced_image_fragment_info.computed_inline_size()
            }
            SpecificFragmentInfo::ScannedText(ref text_fragment_info) => {
                let (range, run) = (&text_fragment_info.range, &text_fragment_info.run);
                let text_bounds = run.metrics_for_range(range).bounding_box;
                text_bounds.size.width
            }
            SpecificFragmentInfo::TableColumn(_) => {
                panic!("Table column fragments do not have inline_size")
            }
            SpecificFragmentInfo::UnscannedText(_) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            }
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
            word_break::T::normal => {
                // Break at normal word boundaries.
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
    /// strategy. If no characters could fit, returns `None`.
    pub fn truncate_to_inline_size(&self, max_inline_size: Au) -> Option<TruncationResult> {
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

        let mut remaining_inline_size = max_inline_size;
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
                debug_assert!(arc_ptr_eq(&this_info.run, &other_info.run));
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
            SpecificFragmentInfo::ScannedText(_) => {}
        };

        let style = &*self.style;
        let noncontent_inline_size = self.border_padding.inline_start_end();

        match self.specific {
            SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut info) => {
                let block_flow = flow_ref::deref_mut(&mut info.flow_ref).as_mut_block();
                block_flow.base.position.size.inline =
                    block_flow.base.intrinsic_inline_sizes.preferred_inline_size;

                // This is a hypothetical box, so it takes up no space.
                self.border_box.size.inline = Au(0);
            }
            SpecificFragmentInfo::InlineBlock(ref mut info) => {
                let block_flow = flow_ref::deref_mut(&mut info.flow_ref).as_mut_block();
                self.border_box.size.inline =
                    max(block_flow.base.intrinsic_inline_sizes.minimum_inline_size,
                        block_flow.base.intrinsic_inline_sizes.preferred_inline_size);
                block_flow.base.block_container_inline_size = self.border_box.size.inline;
                block_flow.base.block_container_writing_mode = self.style.writing_mode;
            }
            SpecificFragmentInfo::InlineAbsolute(ref mut info) => {
                let block_flow = flow_ref::deref_mut(&mut info.flow_ref).as_mut_block();
                self.border_box.size.inline =
                    max(block_flow.base.intrinsic_inline_sizes.minimum_inline_size,
                        block_flow.base.intrinsic_inline_sizes.preferred_inline_size);
                block_flow.base.block_container_inline_size = self.border_box.size.inline;
                block_flow.base.block_container_writing_mode = self.style.writing_mode;
            }
            SpecificFragmentInfo::ScannedText(ref info) => {
                // Scanned text fragments will have already had their content inline-sizes assigned
                // by this point.
                self.border_box.size.inline = info.content_size.inline + noncontent_inline_size
            }
            SpecificFragmentInfo::Image(ref mut image_fragment_info) => {
                let fragment_inline_size = image_fragment_info.image_inline_size();
                let fragment_block_size = image_fragment_info.image_block_size();
                self.border_box.size.inline =
                    image_fragment_info.replaced_image_fragment_info
                                       .calculate_replaced_inline_size(style,
                                                                       noncontent_inline_size,
                                                                       container_inline_size,
                                                                       container_block_size,
                                                                       fragment_inline_size,
                                                                       fragment_block_size);
            }
            SpecificFragmentInfo::Canvas(ref mut canvas_fragment_info) => {
                let fragment_inline_size = canvas_fragment_info.canvas_inline_size();
                let fragment_block_size = canvas_fragment_info.canvas_block_size();
                self.border_box.size.inline =
                    canvas_fragment_info.replaced_image_fragment_info
                                        .calculate_replaced_inline_size(style,
                                                                        noncontent_inline_size,
                                                                        container_inline_size,
                                                                        container_block_size,
                                                                        fragment_inline_size,
                                                                        fragment_block_size);
            }
            SpecificFragmentInfo::Iframe(ref iframe_fragment_info) => {
                self.border_box.size.inline =
                    iframe_fragment_info.calculate_replaced_inline_size(style,
                                                                        container_inline_size) +
                                              noncontent_inline_size;
            }
            _ => panic!("this case should have been handled above"),
        }
    }

    /// Assign block-size for this fragment if it is replaced content. The inline-size must have
    /// been assigned first.
    ///
    /// Ideally, this should follow CSS 2.1 § 10.6.2.
    pub fn assign_replaced_block_size_if_necessary(&mut self, containing_block_block_size: Option<Au>) {
        match self.specific {
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
            SpecificFragmentInfo::ScannedText(_) => {}
        }

        let style = &*self.style;
        let noncontent_block_size = self.border_padding.block_start_end();

        match self.specific {
            SpecificFragmentInfo::Image(ref mut image_fragment_info) => {
                let fragment_inline_size = image_fragment_info.image_inline_size();
                let fragment_block_size = image_fragment_info.image_block_size();
                self.border_box.size.block =
                    image_fragment_info.replaced_image_fragment_info
                                       .calculate_replaced_block_size(style,
                                                                      noncontent_block_size,
                                                                      containing_block_block_size,
                                                                      fragment_inline_size,
                                                                      fragment_block_size);
            }
            SpecificFragmentInfo::Canvas(ref mut canvas_fragment_info) => {
                let fragment_inline_size = canvas_fragment_info.canvas_inline_size();
                let fragment_block_size = canvas_fragment_info.canvas_block_size();
                self.border_box.size.block =
                    canvas_fragment_info.replaced_image_fragment_info
                                        .calculate_replaced_block_size(style,
                                                                       noncontent_block_size,
                                                                       containing_block_block_size,
                                                                       fragment_inline_size,
                                                                       fragment_block_size);
            }
            SpecificFragmentInfo::ScannedText(ref info) => {
                // Scanned text fragments' content block-sizes are calculated by the text run
                // scanner during flow construction.
                self.border_box.size.block = info.content_size.block + noncontent_block_size
            }
            SpecificFragmentInfo::InlineBlock(ref mut info) => {
                // Not the primary fragment, so we do not take the noncontent size into account.
                let block_flow = flow_ref::deref_mut(&mut info.flow_ref).as_block();
                self.border_box.size.block = block_flow.base.position.size.block +
                    block_flow.fragment.margin.block_start_end()
            }
            SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut info) => {
                // Not the primary fragment, so we do not take the noncontent size into account.
                let block_flow = flow_ref::deref_mut(&mut info.flow_ref).as_block();
                self.border_box.size.block = block_flow.base.position.size.block;
            }
            SpecificFragmentInfo::InlineAbsolute(ref mut info) => {
                // Not the primary fragment, so we do not take the noncontent size into account.
                let block_flow = flow_ref::deref_mut(&mut info.flow_ref).as_block();
                self.border_box.size.block = block_flow.base.position.size.block +
                    block_flow.fragment.margin.block_start_end()
            }
            SpecificFragmentInfo::Iframe(ref info) => {
                self.border_box.size.block =
                    info.calculate_replaced_block_size(style, containing_block_block_size) +
                    noncontent_block_size;
            }
            _ => panic!("should have been handled above"),
        }
    }

    /// Calculates block-size above baseline, depth below baseline, and ascent for this fragment
    /// when used in an inline formatting context. See CSS 2.1 § 10.8.1.
    pub fn inline_metrics(&self, layout_context: &LayoutContext) -> InlineMetrics {
        match self.specific {
            SpecificFragmentInfo::Image(ref image_fragment_info) => {
                let computed_block_size = image_fragment_info.replaced_image_fragment_info
                                                             .computed_block_size();
                InlineMetrics {
                    block_size_above_baseline: computed_block_size +
                                                   self.border_padding.block_start,
                    depth_below_baseline: self.border_padding.block_end,
                    ascent: computed_block_size + self.border_padding.block_start,
                }
            }
            SpecificFragmentInfo::Canvas(ref canvas_fragment_info) => {
                let computed_block_size = canvas_fragment_info.replaced_image_fragment_info
                                                              .computed_block_size();
                InlineMetrics {
                    block_size_above_baseline: computed_block_size +
                                                   self.border_padding.block_start,
                    depth_below_baseline: self.border_padding.block_end,
                    ascent: computed_block_size + self.border_padding.block_start,
                }
            }
            SpecificFragmentInfo::ScannedText(ref info) => {
                // Fragments with no glyphs don't contribute any inline metrics.
                // TODO: Filter out these fragments during flow construction?
                if info.insertion_point.is_none() && info.content_size.inline == Au(0) {
                    return InlineMetrics::new(Au(0), Au(0), Au(0));
                }
                // See CSS 2.1 § 10.8.1.
                let line_height = self.calculate_line_height(layout_context);
                let font_derived_metrics =
                    InlineMetrics::from_font_metrics(&info.run.font_metrics, line_height);
                InlineMetrics {
                    block_size_above_baseline: font_derived_metrics.block_size_above_baseline +
                                                   self.border_padding.block_start,
                    depth_below_baseline: font_derived_metrics.depth_below_baseline +
                        self.border_padding.block_end,
                    ascent: font_derived_metrics.ascent + self.border_padding.block_start,
                }
            }
            SpecificFragmentInfo::InlineBlock(ref info) => {
                // See CSS 2.1 § 10.8.1.
                let flow = &info.flow_ref;
                let block_flow = flow.as_block();
                let baseline_offset = match flow.baseline_offset_of_last_line_box_in_flow() {
                    Some(baseline_offset) => baseline_offset,
                    None => block_flow.fragment.border_box.size.block,
                };
                let start_margin = block_flow.fragment.margin.block_start;
                let end_margin = block_flow.fragment.margin.block_end;
                let depth_below_baseline = flow::base(&**flow).position.size.block -
                    baseline_offset + end_margin;
                InlineMetrics::new(baseline_offset + start_margin,
                                   depth_below_baseline,
                                   baseline_offset)
            }
            SpecificFragmentInfo::InlineAbsoluteHypothetical(_) |
            SpecificFragmentInfo::InlineAbsolute(_) => {
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
                        self.text_decoration() != other.text_decoration() ||
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
            SpecificFragmentInfo::Table |
            SpecificFragmentInfo::TableCell |
            SpecificFragmentInfo::TableColumn(_) |
            SpecificFragmentInfo::TableRow |
            SpecificFragmentInfo::Multicol |
            SpecificFragmentInfo::UnscannedText(_) => true,
        }
    }

    /// Determines the inline sizes of inline-block fragments. These cannot be fully computed until
    /// inline size assignment has run for the child flow: thus it is computed "late", during
    /// block size assignment.
    pub fn update_late_computed_replaced_inline_size_if_necessary(&mut self) {
        if let SpecificFragmentInfo::InlineBlock(ref mut inline_block_info) = self.specific {
            let block_flow = flow_ref::deref_mut(&mut inline_block_info.flow_ref).as_block();
            let margin = block_flow.fragment.style.logical_margin();
            self.border_box.size.inline = block_flow.fragment.border_box.size.inline +
                MaybeAuto::from_style(margin.inline_start, Au(0)).specified_or_zero() +
                MaybeAuto::from_style(margin.inline_end, Au(0)).specified_or_zero()
        }
    }

    pub fn update_late_computed_inline_position_if_necessary(&mut self) {
        if let SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut info) = self.specific {
            let position = self.border_box.start.i;
            flow_ref::deref_mut(&mut info.flow_ref)
                .update_late_computed_inline_position_if_necessary(position)
        }
    }

    pub fn update_late_computed_block_position_if_necessary(&mut self) {
        if let SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut info) = self.specific {
            let position = self.border_box.start.b;
            flow_ref::deref_mut(&mut info.flow_ref)
                .update_late_computed_block_position_if_necessary(position)
        }
    }

    pub fn repair_style(&mut self, new_style: &Arc<ServoComputedValues>) {
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
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::UnscannedText(_) => return false,
            _ => {}
        }

        if self.flags.contains(HAS_LAYER) {
            return true
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
        if self.style().get_effects().transform.0.is_some() {
            return true
        }
        match self.style().get_used_transform_style() {
            transform_style::T::flat | transform_style::T::preserve_3d => {
                return true
            }
            transform_style::T::auto => {}
        }

        // FIXME(pcwalton): Don't unconditionally form stacking contexts for `overflow_x: scroll`
        // and `overflow_y: scroll`. This needs multiple layers per stacking context.
        match (self.style().get_box().position,
               self.style().get_position().z_index,
               self.style().get_box().overflow_x,
               self.style().get_box().overflow_y.0) {
            (position::T::absolute,
             z_index::T::Auto,
             overflow_x::T::visible,
             overflow_x::T::visible) |
            (position::T::fixed,
             z_index::T::Auto,
             overflow_x::T::visible,
             overflow_x::T::visible) |
            (position::T::relative,
             z_index::T::Auto,
             overflow_x::T::visible,
             overflow_x::T::visible) => false,
            (position::T::absolute, _, _, _) |
            (position::T::fixed, _, _, _) |
            (position::T::relative, _, _, _) |
            (_, _, overflow_x::T::auto, _) |
            (_, _, overflow_x::T::scroll, _) |
            (_, _, _, overflow_x::T::auto) |
            (_, _, _, overflow_x::T::scroll) => true,
            (position::T::static_, _, _, _) => false
        }
    }

    // Get the effective z-index of this fragment. Z-indices only apply to positioned element
    // per CSS 2 9.9.1 (http://www.w3.org/TR/CSS2/visuren.html#z-index), so this value may differ
    // from the value specified in the style.
    pub fn effective_z_index(&self) -> i32 {
        match self.style().get_box().position {
            position::T::static_ => {},
            _ => return self.style().get_position().z_index.number_or_zero(),
        }

        if self.style().get_effects().transform.0.is_some() {
            return self.style().get_position().z_index.number_or_zero();
        }

        match self.style().get_box().display {
            display::T::flex => self.style().get_position().z_index.number_or_zero(),
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
            SpecificFragmentInfo::ScannedText(ref scanned_text) => {
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
            SpecificFragmentInfo::ScannedText(ref mut scanned_text_fragment_info) => {
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
            SpecificFragmentInfo::ScannedText(ref mut scanned_text_fragment_info) => {
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


    pub fn fragment_id(&self) -> usize {
        return self as *const Fragment as usize;
    }

    pub fn fragment_type(&self) -> FragmentType {
        match self.pseudo {
            PseudoElementType::Normal => FragmentType::FragmentBody,
            PseudoElementType::Before(_) => FragmentType::BeforePseudoContent,
            PseudoElementType::After(_) => FragmentType::AfterPseudoContent,
            PseudoElementType::DetailsSummary(_) => FragmentType::FragmentBody,
            PseudoElementType::DetailsContent(_) => FragmentType::FragmentBody,
        }
    }

    pub fn layer_id(&self) -> LayerId {
        let layer_type = match self.pseudo {
            PseudoElementType::Normal => LayerType::FragmentBody,
            PseudoElementType::Before(_) => LayerType::BeforePseudoContent,
            PseudoElementType::After(_) => LayerType::AfterPseudoContent,
            PseudoElementType::DetailsSummary(_) => LayerType::FragmentBody,
            PseudoElementType::DetailsContent(_) => LayerType::FragmentBody,
        };
        LayerId::new_of_type(layer_type, self.node.id() as usize)
    }

    pub fn layer_id_for_overflow_scroll(&self) -> LayerId {
        LayerId::new_of_type(LayerType::OverflowScroll, self.node.id() as usize)
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
            SpecificFragmentInfo::UnscannedText(_) => true
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
        /// Whether this fragment has a layer.
        const HAS_LAYER = 0x01,
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
impl Encodable for DebugId {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_str(&format!("{:p}", &self))
    }
}

#[cfg(debug_assertions)]
impl Encodable for DebugId {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_u16(self.0)
    }
}
