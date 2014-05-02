/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Box` type, which represents the leaves of the layout tree.

use css::node_style::StyledNode;
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, DisplayListBuildingInfo, ToGfxColor};
use layout::floats::{ClearBoth, ClearLeft, ClearRight, ClearType};
use layout::flow::{Flow, FlowFlagsInfo};
use layout::flow;
use layout::model::{Auto, IntrinsicWidths, MaybeAuto, Specified, specified};
use layout::model;
use layout::util::OpaqueNodeMethods;
use layout::wrapper::{TLayoutNode, ThreadSafeLayoutNode};

use sync::{Arc, Mutex};
use geom::{Point2D, Rect, Size2D, SideOffsets2D};
use geom::approxeq::ApproxEq;
use gfx::color::rgb;
use gfx::display_list::{BackgroundAndBorderLevel, BaseDisplayItem, BorderDisplayItem};
use gfx::display_list::{BorderDisplayItemClass, ClipDisplayItem, ClipDisplayItemClass};
use gfx::display_list::{DisplayList, ImageDisplayItem, ImageDisplayItemClass, LineDisplayItem};
use gfx::display_list::{LineDisplayItemClass, OpaqueNode, SolidColorDisplayItem};
use gfx::display_list::{SolidColorDisplayItemClass, StackingContext, TextDisplayItem};
use gfx::display_list::{TextDisplayItemClass, TextDisplayItemFlags};
use gfx::font::FontStyle;
use gfx::text::text_run::TextRun;
use servo_msg::constellation_msg::{ConstellationChan, FrameRectMsg, PipelineId, SubpageId};
use servo_net::image::holder::ImageHolder;
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::range::*;
use servo_util::namespace;
use servo_util::smallvec::{SmallVec, SmallVec0};
use servo_util::str::is_whitespace;
use std::cast;
use std::cell::RefCell;
use std::from_str::FromStr;
use std::num::Zero;
use style::{ComputedValues, TElement, TNode, cascade, initial_values};
use style::computed_values::{LengthOrPercentage, LengthOrPercentageOrAuto, overflow, LPA_Auto};
use style::computed_values::{background_attachment, background_repeat, border_style, clear};
use style::computed_values::{font_family, line_height, position, text_align, text_decoration};
use style::computed_values::{vertical_align, visibility, white_space};
use url::Url;

/// Boxes (`struct Box`) are the leaves of the layout tree. They cannot position themselves. In
/// general, boxes do not have a simple correspondence with CSS boxes in the specification:
///
/// * Several boxes may correspond to the same CSS box or DOM node. For example, a CSS text box
/// broken across two lines is represented by two boxes.
///
/// * Some CSS boxes are not created at all, such as some anonymous block boxes induced by inline
///   boxes with block-level sibling boxes. In that case, Servo uses an `InlineFlow` with
///   `BlockFlow` siblings; the `InlineFlow` is block-level, but not a block container. It is
///   positioned as if it were a block box, but its children are positioned according to inline
///   flow.
///
/// A `GenericBox` is an empty box that contributes only borders, margins, padding, and
/// backgrounds. It is analogous to a CSS nonreplaced content box.
///
/// A box's type influences how its styles are interpreted during layout. For example, replaced
/// content such as images are resized differently from tables, text, or other content. Different
/// types of boxes may also contain custom data; for example, text boxes contain text.
///
/// FIXME(pcwalton): This can be slimmed down quite a bit.
#[deriving(Clone)]
pub struct Box {
    /// An opaque reference to the DOM node that this `Box` originates from.
    pub node: OpaqueNode,

    /// The CSS style of this box.
    pub style: Arc<ComputedValues>,

    /// The position of this box relative to its owning flow.
    /// The size includes padding and border, but not margin.
    pub border_box: RefCell<Rect<Au>>,

    /// The border of the content box.
    ///
    /// FIXME(pcwalton): This need not be stored in the box.
    pub border: RefCell<SideOffsets2D<Au>>,

    /// The padding of the content box.
    pub padding: RefCell<SideOffsets2D<Au>>,

    /// The margin of the content box.
    pub margin: RefCell<SideOffsets2D<Au>>,

    /// Info specific to the kind of box. Keep this enum small.
    pub specific: SpecificBoxInfo,

    /// positioned box offsets
    pub position_offsets: RefCell<SideOffsets2D<Au>>,

    /// Inline data
    pub inline_info: RefCell<Option<InlineInfo>>,

    /// New-line chracter(\n)'s positions(relative, not absolute)
    pub new_line_pos: ~[uint],
}

/// Info specific to the kind of box. Keep this enum small.
#[deriving(Clone)]
pub enum SpecificBoxInfo {
    GenericBox,
    ImageBox(ImageBoxInfo),
    IframeBox(IframeBoxInfo),
    ScannedTextBox(ScannedTextBoxInfo),
    TableBox,
    TableCellBox,
    TableColumnBox(TableColumnBoxInfo),
    TableRowBox,
    TableWrapperBox,
    UnscannedTextBox(UnscannedTextBoxInfo),
}

/// A box that represents a replaced content image and its accompanying borders, shadows, etc.
#[deriving(Clone)]
pub struct ImageBoxInfo {
    /// The image held within this box.
    pub image: RefCell<ImageHolder>,
    pub computed_width: RefCell<Option<Au>>,
    pub computed_height: RefCell<Option<Au>>,
    pub dom_width: Option<Au>,
    pub dom_height: Option<Au>,
}

impl ImageBoxInfo {
    /// Creates a new image box from the given URL and local image cache.
    ///
    /// FIXME(pcwalton): The fact that image boxes store the cache in the box makes little sense to
    /// me.
    pub fn new(node: &ThreadSafeLayoutNode,
               image_url: Url,
               local_image_cache: Arc<Mutex<*()>>)
               -> ImageBoxInfo {
        fn convert_length(node: &ThreadSafeLayoutNode, name: &str) -> Option<Au> {
            let element = node.as_element();
            element.get_attr(&namespace::Null, name).and_then(|string| {
                let n: Option<int> = FromStr::from_str(string);
                n
            }).and_then(|pixels| Some(Au::from_px(pixels)))
        }

        ImageBoxInfo {
            image: RefCell::new(ImageHolder::new(image_url, local_image_cache)),
            computed_width: RefCell::new(None),
            computed_height: RefCell::new(None),
            dom_width: convert_length(node,"width"),
            dom_height: convert_length(node,"height"),
        }
    }

    /// Returns the calculated width of the image, accounting for the width attribute.
    pub fn computed_width(&self) -> Au {
        match &*self.computed_width.borrow() {
            &Some(width) => {
                width
            },
            &None => {
                fail!("image width is not computed yet!");
            }
        }
    }
    /// Returns width of image(just original width)
    pub fn image_width(&self) -> Au {
        let mut image_ref = self.image.borrow_mut();
        Au::from_px(image_ref.get_size().unwrap_or(Size2D(0,0)).width)
    }

    // Return used value for width or height.
    //
    // `dom_length`: width or height as specified in the `img` tag.
    // `style_length`: width as given in the CSS
    pub fn style_length(style_length: LengthOrPercentageOrAuto,
                        dom_length: Option<Au>,
                        container_width: Au) -> MaybeAuto {
        match (MaybeAuto::from_style(style_length,container_width),dom_length) {
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
    /// Returns the calculated height of the image, accounting for the height attribute.
    pub fn computed_height(&self) -> Au {
        match &*self.computed_height.borrow() {
            &Some(height) => {
                height
            },
            &None => {
                fail!("image height is not computed yet!");
            }
        }
    }

    /// Returns height of image(just original height)
    pub fn image_height(&self) -> Au {
        let mut image_ref = self.image.borrow_mut();
        Au::from_px(image_ref.get_size().unwrap_or(Size2D(0,0)).height)
    }
}

/// A box that represents an inline frame (iframe). This stores the pipeline ID so that the size
/// of this iframe can be communicated via the constellation to the iframe's own layout task.
#[deriving(Clone)]
pub struct IframeBoxInfo {
    /// The pipeline ID of this iframe.
    pub pipeline_id: PipelineId,
    /// The subpage ID of this iframe.
    pub subpage_id: SubpageId,
}

impl IframeBoxInfo {
    /// Creates the information specific to an iframe box.
    pub fn new(node: &ThreadSafeLayoutNode) -> IframeBoxInfo {
        let (pipeline_id, subpage_id) = node.iframe_pipeline_and_subpage_ids();
        IframeBoxInfo {
            pipeline_id: pipeline_id,
            subpage_id: subpage_id,
        }
    }
}

/// A scanned text box represents a single run of text with a distinct style. A `TextBox` may be
/// split into two or more boxes across line breaks. Several `TextBox`es may correspond to a single
/// DOM text node. Split text boxes are implemented by referring to subsets of a single `TextRun`
/// object.
#[deriving(Clone)]
pub struct ScannedTextBoxInfo {
    /// The text run that this represents.
    pub run: Arc<~TextRun>,

    /// The range within the above text run that this represents.
    pub range: Range,
}

impl ScannedTextBoxInfo {
    /// Creates the information specific to a scanned text box from a range and a text run.
    pub fn new(run: Arc<~TextRun>, range: Range) -> ScannedTextBoxInfo {
        ScannedTextBoxInfo {
            run: run,
            range: range,
        }
    }
}

/// Data for an unscanned text box. Unscanned text boxes are the results of flow construction that
/// have not yet had their width determined.
#[deriving(Clone)]
pub struct UnscannedTextBoxInfo {
    /// The text inside the box.
    pub text: ~str,
}

impl UnscannedTextBoxInfo {
    /// Creates a new instance of `UnscannedTextBoxInfo` from the given DOM node.
    pub fn new(node: &ThreadSafeLayoutNode) -> UnscannedTextBoxInfo {
        // FIXME(pcwalton): Don't copy text; atomically reference count it instead.
        UnscannedTextBoxInfo {
            text: node.text(),
        }
    }

    /// Creates a new instance of `UnscannedTextBoxInfo` from the given text.
    #[inline]
    pub fn from_text(text: ~str) -> UnscannedTextBoxInfo {
        UnscannedTextBoxInfo {
            text: text,
        }
    }
}

/// Represents the outcome of attempting to split a box.
pub enum SplitBoxResult {
    CannotSplit,
    // in general, when splitting the left or right side can
    // be zero length, due to leading/trailing trimmable whitespace
    SplitDidFit(Option<Box>, Option<Box>),
    SplitDidNotFit(Option<Box>, Option<Box>)
}


/// Data for inline boxes.
///
/// FIXME(#2013, pcwalton): Copying `InlineParentInfo` vectors all the time is really inefficient.
/// Use atomic reference counting instead.
#[deriving(Clone)]
pub struct InlineInfo {
    pub parent_info: SmallVec0<InlineParentInfo>,
    pub baseline: Au,
}

impl InlineInfo {
    pub fn new() -> InlineInfo {
        InlineInfo {
            parent_info: SmallVec0::new(),
            baseline: Au(0),
        }
    }
}

#[deriving(Clone)]
pub struct InlineParentInfo {
    pub padding: SideOffsets2D<Au>,
    pub border: SideOffsets2D<Au>,
    pub margin: SideOffsets2D<Au>,
    pub style: Arc<ComputedValues>,
    pub font_ascent: Au,
    pub font_descent: Au,
    pub node: OpaqueNode,
}

/// A box that represents a table column.
#[deriving(Clone)]
pub struct TableColumnBoxInfo {
    /// the number of columns a <col> element should span
    pub span: Option<int>,
}

impl TableColumnBoxInfo {
    /// Create the information specific to an table column box.
    pub fn new(node: &ThreadSafeLayoutNode) -> TableColumnBoxInfo {
        let span = {
            let element = node.as_element();
            element.get_attr(&namespace::Null, "span").and_then(|string| {
                let n: Option<int> = FromStr::from_str(string);
                n
            })
        };
        TableColumnBoxInfo {
            span: span,
        }
    }
}

// FIXME: Take just one parameter and use concat_ident! (mozilla/rust#12249)
macro_rules! def_noncontent( ($side:ident, $get:ident, $inline_get:ident) => (
    impl Box {
        pub fn $get(&self) -> Au {
            self.border.borrow().$side + self.padding.borrow().$side
        }

        pub fn $inline_get(&self) -> Au {
            let mut val = Au::new(0);
            let info = self.inline_info.borrow();
            match &*info {
                &Some(ref info) => {
                    for info in info.parent_info.iter() {
                        val = val + info.border.$side + info.padding.$side;
                    }
                },
                &None => {}
            }
            val
        }
    }
))

macro_rules! def_noncontent_horiz( ($side:ident, $merge:ident, $clear:ident) => (
    impl Box {
        pub fn $merge(&self, other_box: &Box) {
            let mut info = self.inline_info.borrow_mut();
            let other_info = other_box.inline_info.borrow();

            match &*other_info {
                &Some(ref other_info) => {
                    match &mut *info {
                        &Some(ref mut info) => {
                            for other_item in other_info.parent_info.iter() {
                                for item in info.parent_info.mut_iter() {
                                    if item.node == other_item.node {
                                        item.border.$side = other_item.border.$side;
                                        item.padding.$side = other_item.padding.$side;
                                        item.margin.$side = other_item.margin.$side;
                                        break;
                                    }
                                }
                            }
                        },
                        &None => {}
                    }
                },
                &None => {}
            }
        }

        pub fn $clear(&self) {
            let mut info = self.inline_info.borrow_mut();
            match &mut *info {
                &Some(ref mut info) => {
                    for item in info.parent_info.mut_iter() {
                        item.border.$side = Au::new(0);
                        item.padding.$side = Au::new(0);
                        item.margin.$side = Au::new(0);
                    }
                },
                &None => {}
            }
        }
    }
))

def_noncontent!(left,   noncontent_left,   noncontent_inline_left)
def_noncontent!(right,  noncontent_right,  noncontent_inline_right)
def_noncontent!(top,    noncontent_top,    noncontent_inline_top)
def_noncontent!(bottom, noncontent_bottom, noncontent_inline_bottom)

def_noncontent_horiz!(left,  merge_noncontent_inline_left,  clear_noncontent_inline_left)
def_noncontent_horiz!(right, merge_noncontent_inline_right, clear_noncontent_inline_right)

impl Box {
    /// Constructs a new `Box` instance for the given node.
    ///
    /// Arguments:
    ///
    ///   * `constructor`: The flow constructor.
    ///
    ///   * `node`: The node to create a box for.
    pub fn new(constructor: &mut FlowConstructor, node: &ThreadSafeLayoutNode) -> Box {
        Box {
            node: OpaqueNodeMethods::from_thread_safe_layout_node(node),
            style: node.style().clone(),
            border_box: RefCell::new(Rect::zero()),
            border: RefCell::new(Zero::zero()),
            padding: RefCell::new(Zero::zero()),
            margin: RefCell::new(Zero::zero()),
            specific: constructor.build_specific_box_info_for_node(node),
            position_offsets: RefCell::new(Zero::zero()),
            inline_info: RefCell::new(None),
            new_line_pos: ~[],
        }
    }

    /// Constructs a new `Box` instance from a specific info.
    pub fn new_from_specific_info(node: &ThreadSafeLayoutNode, specific: SpecificBoxInfo) -> Box {
        Box {
            node: OpaqueNodeMethods::from_thread_safe_layout_node(node),
            style: node.style().clone(),
            border_box: RefCell::new(Rect::zero()),
            border: RefCell::new(Zero::zero()),
            padding: RefCell::new(Zero::zero()),
            margin: RefCell::new(Zero::zero()),
            specific: specific,
            position_offsets: RefCell::new(Zero::zero()),
            inline_info: RefCell::new(None),
            new_line_pos: ~[],
        }
    }

    /// Constructs a new `Box` instance for an anonymous table object.
    pub fn new_anonymous_table_box(node: &ThreadSafeLayoutNode, specific: SpecificBoxInfo) -> Box {
        // CSS 2.1 § 17.2.1 This is for non-inherited properties on anonymous table boxes
        // example:
        //
        //     <div style="display: table">
        //         Foo
        //     </div>
        //
        // Anonymous table boxes, TableRowBox and TableCellBox, are generated around `Foo`, but it shouldn't inherit the border.

        let (node_style, _) = cascade(&[], false, Some(&**node.style()),
                                      &initial_values(), None);
        Box {
            node: OpaqueNodeMethods::from_thread_safe_layout_node(node),
            style: Arc::new(node_style),
            border_box: RefCell::new(Rect::zero()),
            border: RefCell::new(Zero::zero()),
            padding: RefCell::new(Zero::zero()),
            margin: RefCell::new(Zero::zero()),
            specific: specific,
            position_offsets: RefCell::new(Zero::zero()),
            inline_info: RefCell::new(None),
            new_line_pos: ~[],
        }
    }

    /// Constructs a new `Box` instance from an opaque node.
    pub fn from_opaque_node_and_style(node: OpaqueNode,
                                      style: Arc<ComputedValues>,
                                      specific: SpecificBoxInfo)
                                      -> Box {
        Box {
            node: node,
            style: style,
            border_box: RefCell::new(Rect::zero()),
            border: RefCell::new(Zero::zero()),
            padding: RefCell::new(Zero::zero()),
            margin: RefCell::new(Zero::zero()),
            specific: specific,
            position_offsets: RefCell::new(Zero::zero()),
            inline_info: RefCell::new(None),
            new_line_pos: ~[],
        }
    }

    /// Returns a debug ID of this box. This ID should not be considered stable across multiple
    /// layouts or box manipulations.
    pub fn debug_id(&self) -> uint {
        unsafe {
            cast::transmute(self)
        }
    }

    /// Transforms this box into another box of the given type, with the given size, preserving all
    /// the other data.
    pub fn transform(&self, size: Size2D<Au>, specific: SpecificBoxInfo) -> Box {
        Box {
            node: self.node,
            style: self.style.clone(),
            border_box: RefCell::new(Rect(self.border_box.borrow().origin, size)),
            border: RefCell::new(*self.border.borrow()),
            padding: RefCell::new(*self.padding.borrow()),
            margin: RefCell::new(*self.margin.borrow()),
            specific: specific,
            position_offsets: RefCell::new(Zero::zero()),
            inline_info: self.inline_info.clone(),
            new_line_pos: self.new_line_pos.clone(),
        }
    }

    /// Uses the style only to estimate the intrinsic widths. These may be modified for text or
    /// replaced elements.
    fn style_specified_intrinsic_width(&self) -> IntrinsicWidths {
        let (use_margins, use_padding) = match self.specific {
            GenericBox | IframeBox(_) | ImageBox(_) => (true, true),
            TableBox | TableCellBox => (false, true),
            TableWrapperBox => (true, false),
            TableRowBox => (false, false),
            ScannedTextBox(_) | TableColumnBox(_) | UnscannedTextBox(_) => {
                // Styles are irrelevant for these kinds of boxes.
                return IntrinsicWidths::new()
            }
        };

        let style = self.style();
        let width = MaybeAuto::from_style(style.Box.get().width, Au::new(0)).specified_or_zero();

        let (margin_left, margin_right) = if use_margins {
            (MaybeAuto::from_style(style.Margin.get().margin_left, Au(0)).specified_or_zero(),
             MaybeAuto::from_style(style.Margin.get().margin_right, Au(0)).specified_or_zero())
        } else {
            (Au(0), Au(0))
        };

        let (padding_left, padding_right) = if use_padding {
            (self.compute_padding_length(style.Padding.get().padding_left, Au(0)),
             self.compute_padding_length(style.Padding.get().padding_right, Au(0)))
        } else {
            (Au(0), Au(0))
        };

        let surround_width = margin_left + margin_right + padding_left + padding_right +
                self.border.borrow().left + self.border.borrow().right;

        IntrinsicWidths {
            minimum_width: width,
            preferred_width: width,
            surround_width: surround_width,
        }
    }

    pub fn calculate_line_height(&self, font_size: Au) -> Au {
        let from_inline = match self.style().InheritedBox.get().line_height {
            line_height::Normal => font_size.scale_by(1.14),
            line_height::Number(l) => font_size.scale_by(l),
            line_height::Length(l) => l
        };
        let minimum = self.style().InheritedBox.get()._servo_minimum_line_height;
        Au::max(from_inline, minimum)
    }

    /// Populates the box model border parameters from the given computed style.
    ///
    /// FIXME(pcwalton): This should not be necessary. Just go to the style.
    pub fn compute_borders(&self, style: &ComputedValues) {
        let border = match self.specific {
            TableWrapperBox => {
                SideOffsets2D::new(Au(0), Au(0), Au(0), Au(0))
            },
            _ => {
                #[inline]
                fn width(width: Au, style: border_style::T) -> Au {
                    if style == border_style::none {
                        Au(0)
                    } else {
                        width
                    }
                }

                SideOffsets2D::new(width(style.Border.get().border_top_width,
                                         style.Border.get().border_top_style),
                                   width(style.Border.get().border_right_width,
                                         style.Border.get().border_right_style),
                                   width(style.Border.get().border_bottom_width,
                                         style.Border.get().border_bottom_style),
                                   width(style.Border.get().border_left_width,
                                         style.Border.get().border_left_style))
            }
        };
        *self.border.borrow_mut() = border;
    }

    pub fn compute_positioned_offsets(&self, style: &ComputedValues, containing_width: Au, containing_height: Au) {
        *self.position_offsets.borrow_mut() = SideOffsets2D::new(
                MaybeAuto::from_style(style.PositionOffsets.get().top, containing_height)
                .specified_or_zero(),
                MaybeAuto::from_style(style.PositionOffsets.get().right, containing_width)
                .specified_or_zero(),
                MaybeAuto::from_style(style.PositionOffsets.get().bottom, containing_height)
                .specified_or_zero(),
                MaybeAuto::from_style(style.PositionOffsets.get().left, containing_width)
                .specified_or_zero());
    }

    /// Compute and set margin-top and margin-bottom values.
    ///
    /// If a value is specified or is a percentage, we calculate the right value here.
    /// If it is auto, it is up to assign-height to ignore this value and
    /// calculate the correct margin values.
    pub fn compute_margin_top_bottom(&self, containing_block_width: Au) {
        match self.specific {
            TableBox | TableCellBox | TableRowBox | TableColumnBox(_) => {
                *self.margin.borrow_mut() = SideOffsets2D::new(Au(0), Au(0), Au(0), Au(0));
            },
            _ => {
                let style = self.style();
                // Note: CSS 2.1 defines margin % values wrt CB *width* (not height).
                let margin_top = MaybeAuto::from_style(style.Margin.get().margin_top,
                                                       containing_block_width).specified_or_zero();
                let margin_bottom = MaybeAuto::from_style(style.Margin.get().margin_bottom,
                                                          containing_block_width).specified_or_zero();
                let mut margin = *self.margin.borrow();
                margin.top = margin_top;
                margin.bottom = margin_bottom;
                *self.margin.borrow_mut() = margin;
            }
        }
    }

    /// Populates the box model padding parameters from the given computed style.
    pub fn compute_padding(&self, style: &ComputedValues, containing_block_width: Au) {
        let padding = match self.specific {
            TableColumnBox(_) | TableRowBox | TableWrapperBox => {
                SideOffsets2D::new(Au(0), Au(0), Au(0), Au(0))
            },
            GenericBox | IframeBox(_) | ImageBox(_) | TableBox | TableCellBox |
            ScannedTextBox(_) | UnscannedTextBox(_) => {
                SideOffsets2D::new(self.compute_padding_length(style.Padding
                                                                    .get()
                                                                    .padding_top,
                                                               containing_block_width),
                                   self.compute_padding_length(style.Padding
                                                                    .get()
                                                                    .padding_right,
                                                               containing_block_width),
                                   self.compute_padding_length(style.Padding
                                                                    .get()
                                                                    .padding_bottom,
                                                               containing_block_width),
                                   self.compute_padding_length(style.Padding
                                                                    .get()
                                                                    .padding_left,
                                                               containing_block_width))
            }
        };
        *self.padding.borrow_mut() = padding;
    }

    fn compute_padding_length(&self, padding: LengthOrPercentage, content_box_width: Au) -> Au {
        specified(padding, content_box_width)
    }

    pub fn padding_box_size(&self) -> Size2D<Au> {
        let border_box_size = self.border_box.borrow().size;
        Size2D(border_box_size.width - self.border.borrow().left - self.border.borrow().right,
               border_box_size.height - self.border.borrow().top - self.border.borrow().bottom)
    }

    pub fn noncontent_width(&self) -> Au {
        self.noncontent_left() + self.noncontent_right()
    }

    pub fn noncontent_height(&self) -> Au {
        self.noncontent_top() + self.noncontent_bottom()
    }

    // Return offset from original position because of `position: relative`.
    pub fn relative_position(&self, container_block_size: &Size2D<Au>) -> Point2D<Au> {
        fn left_right(style: &ComputedValues, block_width: Au) -> Au {
            // TODO(ksh8281) : consider RTL(right-to-left) culture
            match (style.PositionOffsets.get().left, style.PositionOffsets.get().right) {
                (LPA_Auto, _) => {
                    -MaybeAuto::from_style(style.PositionOffsets.get().right, block_width)
                        .specified_or_zero()
                }
                (_, _) => {
                    MaybeAuto::from_style(style.PositionOffsets.get().left, block_width)
                        .specified_or_zero()
                }
            }
        }

        fn top_bottom(style: &ComputedValues,block_height: Au) -> Au {
            match (style.PositionOffsets.get().top, style.PositionOffsets.get().bottom) {
                (LPA_Auto, _) => {
                    -MaybeAuto::from_style(style.PositionOffsets.get().bottom, block_height)
                        .specified_or_zero()
                }
                (_, _) => {
                    MaybeAuto::from_style(style.PositionOffsets.get().top, block_height)
                        .specified_or_zero()
                }
            }
        }

        let mut rel_pos: Point2D<Au> = Point2D {
            x: Au::new(0),
            y: Au::new(0),
        };

        if self.style().Box.get().position == position::relative {
            rel_pos.x = rel_pos.x + left_right(self.style(), container_block_size.width);
            rel_pos.y = rel_pos.y + top_bottom(self.style(), container_block_size.height);
        }

        // Go over the ancestor boxes and add all relative offsets (if any).
        let info = self.inline_info.borrow();
        match &*info {
            &Some(ref info) => {
                for info in info.parent_info.iter() {
                    if info.style.Box.get().position == position::relative {
                        rel_pos.x = rel_pos.x + left_right(&*info.style,
                                                           container_block_size.width);
                        rel_pos.y = rel_pos.y + top_bottom(&*info.style,
                                                           container_block_size.height);
                    }
                }
            },
            &None => {}
        }
        rel_pos
    }

    /// Always inline for SCCP.
    ///
    /// FIXME(pcwalton): Just replace with the clear type from the style module for speed?
    #[inline(always)]
    pub fn clear(&self) -> Option<ClearType> {
        let style = self.style();
        match style.Box.get().clear {
            clear::none => None,
            clear::left => Some(ClearLeft),
            clear::right => Some(ClearRight),
            clear::both => Some(ClearBoth),
        }
    }

    /// Converts this node's computed style to a font style used for rendering.
    ///
    /// FIXME(pcwalton): This should not be necessary; just make the font part of style sharable
    /// with the display list somehow. (Perhaps we should use an ARC.)
    pub fn font_style(&self) -> FontStyle {
        let my_style = self.style();

        debug!("(font style) start");

        // FIXME: Too much allocation here.
        let font_families = my_style.Font.get().font_family.iter().map(|family| {
            match *family {
                font_family::FamilyName(ref name) => (*name).clone(),
            }
        }).collect();
        debug!("(font style) font families: `{:?}`", font_families);

        let font_size = my_style.Font.get().font_size.to_f64().unwrap() / 60.0;
        debug!("(font style) font size: `{:f}px`", font_size);

        FontStyle {
            pt_size: font_size,
            weight: my_style.Font.get().font_weight,
            style: my_style.Font.get().font_style,
            families: font_families,
        }
    }

    #[inline(always)]
    pub fn style<'a>(&'a self) -> &'a ComputedValues {
        &*self.style
    }

    /// Returns the text alignment of the computed style of the nearest ancestor-or-self `Element`
    /// node.
    pub fn text_align(&self) -> text_align::T {
        self.style().InheritedText.get().text_align
    }

    pub fn vertical_align(&self) -> vertical_align::T {
        self.style().Box.get().vertical_align
    }

    pub fn white_space(&self) -> white_space::T {
        self.style().InheritedText.get().white_space
    }

    /// Returns the text decoration of this box, according to the style of the nearest ancestor
    /// element.
    ///
    /// NB: This may not be the actual text decoration, because of the override rules specified in
    /// CSS 2.1 § 16.3.1. Unfortunately, computing this properly doesn't really fit into Servo's
    /// model. Therefore, this is a best lower bound approximation, but the end result may actually
    /// have the various decoration flags turned on afterward.
    pub fn text_decoration(&self) -> text_decoration::T {
        self.style().Text.get().text_decoration
    }

    /// Returns the left offset from margin edge to content edge.
    pub fn left_offset(&self) -> Au {
        match self.specific {
            TableWrapperBox => self.margin.borrow().left,
            TableBox | TableCellBox => self.border.borrow().left + self.padding.borrow().left,
            TableRowBox => self.border.borrow().left,
            TableColumnBox(_) => Au(0),
            _ => self.margin.borrow().left + self.border.borrow().left + self.padding.borrow().left
        }
    }

    /// Returns the top offset from margin edge to content edge.
    pub fn top_offset(&self) -> Au {
        match self.specific {
            TableWrapperBox => self.margin.borrow().top,
            TableBox | TableCellBox => self.border.borrow().top + self.padding.borrow().top,
            TableRowBox => self.border.borrow().top,
            TableColumnBox(_) => Au(0),
            _ => self.margin.borrow().top + self.border.borrow().top + self.padding.borrow().top
        }
    }

    /// Returns the bottom offset from margin edge to content edge.
    pub fn bottom_offset(&self) -> Au {
        match self.specific {
            TableWrapperBox => self.margin.borrow().bottom,
            TableBox | TableCellBox => self.border.borrow().bottom + self.padding.borrow().bottom,
            TableRowBox => self.border.borrow().bottom,
            TableColumnBox(_) => Au(0),
            _ => self.margin.borrow().bottom + self.border.borrow().bottom + self.padding.borrow().bottom
        }
    }

    /// Returns true if this element can be split. This is true for text boxes.
    pub fn can_split(&self) -> bool {
        match self.specific {
            ScannedTextBox(..) => true,
            _ => false,
        }
    }

    /// Returns the amount of left and right "fringe" used by this box. This is based on margins,
    /// borders, padding, and width.
    pub fn get_used_width(&self) -> (Au, Au) {
        // TODO: This should actually do some computation! See CSS 2.1, Sections 10.3 and 10.4.
        (Au::new(0), Au::new(0))
    }

    /// Returns the amount of left and right "fringe" used by this box. This should be based on
    /// margins, borders, padding, and width.
    pub fn get_used_height(&self) -> (Au, Au) {
        // TODO: This should actually do some computation! See CSS 2.1, Sections 10.5 and 10.6.
        (Au::new(0), Au::new(0))
    }

    pub fn paint_inline_background_border_if_applicable(&self,
                                                        list: &mut DisplayList,
                                                        absolute_bounds: &Rect<Au>,
                                                        offset: &Point2D<Au>) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a box".
        let info = self.inline_info.borrow();
        match &*info {
            &Some(ref box_info) => {
                let mut bg_rect = absolute_bounds.clone();
                for info in box_info.parent_info.as_slice().rev_iter() {
                    // TODO (ksh8281) compute vertical-align, line-height
                    bg_rect.origin.y = box_info.baseline + offset.y - info.font_ascent;
                    bg_rect.size.height = info.font_ascent + info.font_descent;
                    let background_color = info.style.resolve_color(
                        info.style.Background.get().background_color);

                    if !background_color.alpha.approx_eq(&0.0) {
                        let solid_color_display_item = ~SolidColorDisplayItem {
                            base: BaseDisplayItem {
                                bounds: bg_rect.clone(),
                                node: self.node,
                            },
                            color: background_color.to_gfx_color(),
                        };

                        list.push(SolidColorDisplayItemClass(solid_color_display_item))
                    }

                    let border = &info.border;

                    // Fast path.
                    if border.is_zero() {
                        continue
                    }

                    bg_rect.origin.y = bg_rect.origin.y - border.top;
                    bg_rect.size.height = bg_rect.size.height + border.top + border.bottom;

                    let style = &info.style;
                    let top_color = style.resolve_color(style.Border.get().border_top_color);
                    let right_color = style.resolve_color(style.Border.get().border_right_color);
                    let bottom_color = style.resolve_color(style.Border.get().border_bottom_color);
                    let left_color = style.resolve_color(style.Border.get().border_left_color);
                    let top_style = style.Border.get().border_top_style;
                    let right_style = style.Border.get().border_right_style;
                    let bottom_style = style.Border.get().border_bottom_style;
                    let left_style = style.Border.get().border_left_style;

                    let border_display_item = ~BorderDisplayItem {
                        base: BaseDisplayItem {
                            bounds: bg_rect,
                            node: self.node,
                        },
                        border: border.clone(),
                        color: SideOffsets2D::new(top_color.to_gfx_color(),
                        right_color.to_gfx_color(),
                        bottom_color.to_gfx_color(),
                        left_color.to_gfx_color()),
                        style: SideOffsets2D::new(top_style, right_style, bottom_style, left_style)
                    };

                    list.push(BorderDisplayItemClass(border_display_item));

                    bg_rect.origin.x = bg_rect.origin.x + border.left;
                    bg_rect.size.width = bg_rect.size.width - border.left - border.right;
                }
            },
            &None => {}
        }
    }
    /// Adds the display items necessary to paint the background of this box to the display list if
    /// necessary.
    pub fn paint_background_if_applicable(&self,
                                          list: &mut DisplayList,
                                          builder: &DisplayListBuilder,
                                          absolute_bounds: &Rect<Au>) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a box".
        let style = self.style();
        let background_color = style.resolve_color(style.Background.get().background_color);
        if !background_color.alpha.approx_eq(&0.0) {
            let display_item = ~SolidColorDisplayItem {
                base: BaseDisplayItem {
                    bounds: *absolute_bounds,
                    node: self.node,
                },
                color: background_color.to_gfx_color(),
            };

            list.push(SolidColorDisplayItemClass(display_item))
        }

        // The background image is painted on top of the background color.
        // Implements background image, per spec:
        // http://www.w3.org/TR/CSS21/colors.html#background
        match style.Background.get().background_image {
            Some(ref image_url) => {
                let mut holder = ImageHolder::new(image_url.clone(),
                                                  builder.ctx.image_cache.clone());
                match holder.get_image() {
                    Some(image) => {
                        debug!("(building display list) building background image");

                        // Adjust bounds for `background-position` and `background-attachment`.
                        let mut bounds = *absolute_bounds;
                        let horizontal_position = model::specified(
                            style.Background.get().background_position.horizontal,
                            bounds.size.width);
                        let vertical_position = model::specified(
                            style.Background.get().background_position.vertical,
                            bounds.size.height);

                        let clip_display_item;
                        match style.Background.get().background_attachment {
                            background_attachment::scroll => {
                                clip_display_item = None;
                                bounds.origin.x = bounds.origin.x + horizontal_position;
                                bounds.origin.y = bounds.origin.y + vertical_position;
                                bounds.size.width = bounds.size.width - horizontal_position;
                                bounds.size.height = bounds.size.height - vertical_position;
                            }
                            background_attachment::fixed => {
                                clip_display_item = Some(~ClipDisplayItem {
                                    base: BaseDisplayItem {
                                        bounds: bounds,
                                        node: self.node,
                                    },
                                    child_list: SmallVec0::new(),
                                    need_clip: true,
                                });

                                bounds = Rect {
                                    origin: Point2D(horizontal_position, vertical_position),
                                    size: Size2D(bounds.origin.x + bounds.size.width,
                                                 bounds.origin.y + bounds.size.height),
                                }
                            }
                        }
                        // Adjust sizes for `background-repeat`.
                        match style.Background.get().background_repeat {
                            background_repeat::no_repeat => {
                                bounds.size.width = Au::from_px(image.width as int);
                                bounds.size.height = Au::from_px(image.height as int)
                            }
                            background_repeat::repeat_x => {
                                bounds.size.height = Au::from_px(image.height as int)
                            }
                            background_repeat::repeat_y => {
                                bounds.size.width = Au::from_px(image.width as int)
                            }
                            background_repeat::repeat => {}
                        };


                        // Create the image display item.
                        let image_display_item = ImageDisplayItemClass(~ImageDisplayItem {
                            base: BaseDisplayItem {
                                bounds: bounds,
                                node: self.node,
                            },
                            image: image.clone(),
                            stretch_size: Size2D(Au::from_px(image.width as int),
                                                 Au::from_px(image.height as int)),
                        });

                        match clip_display_item {
                            None => list.push(image_display_item),
                            Some(mut clip_display_item) => {
                                clip_display_item.child_list.push(image_display_item);
                                list.push(ClipDisplayItemClass(clip_display_item))
                            }
                        }
                    }
                    None => {
                        // No image data at all? Do nothing.
                        //
                        // TODO: Add some kind of placeholder background image.
                        debug!("(building display list) no background image :(");
                    }
                }
            }
            None => {}
        }
    }

    /// Adds the display items necessary to paint the borders of this box to a display list if
    /// necessary.
    pub fn paint_borders_if_applicable(&self, list: &mut DisplayList, abs_bounds: &Rect<Au>) {
        // Fast path.
        let border = self.border.borrow();
        if border.is_zero() {
            return
        }

        let style = self.style();
        let top_color = style.resolve_color(style.Border.get().border_top_color);
        let right_color = style.resolve_color(style.Border.get().border_right_color);
        let bottom_color = style.resolve_color(style.Border.get().border_bottom_color);
        let left_color = style.resolve_color(style.Border.get().border_left_color);
        let top_style = style.Border.get().border_top_style;
        let right_style = style.Border.get().border_right_style;
        let bottom_style = style.Border.get().border_bottom_style;
        let left_style = style.Border.get().border_left_style;

        let mut abs_bounds = abs_bounds.clone();
        abs_bounds.origin.x = abs_bounds.origin.x + self.noncontent_inline_left();
        abs_bounds.size.width = abs_bounds.size.width - self.noncontent_inline_left()
            - self.noncontent_inline_right();

        // Append the border to the display list.
        let border_display_item = ~BorderDisplayItem {
            base: BaseDisplayItem {
                bounds: abs_bounds,
                node: self.node,
            },
            border: *border,
            color: SideOffsets2D::new(top_color.to_gfx_color(),
                                      right_color.to_gfx_color(),
                                      bottom_color.to_gfx_color(),
                                      left_color.to_gfx_color()),
            style: SideOffsets2D::new(top_style,
                                      right_style,
                                      bottom_style,
                                      left_style)
        };

        list.push(BorderDisplayItemClass(border_display_item))
    }

    fn build_debug_borders_around_text_boxes(&self,
                                             stacking_context: &mut StackingContext,
                                             flow_origin: Point2D<Au>,
                                             text_box: &ScannedTextBoxInfo) {
        let box_bounds = self.border_box.borrow();
        let absolute_box_bounds = box_bounds.translate(&flow_origin);

        // Compute the text box bounds and draw a border surrounding them.
        let debug_border = SideOffsets2D::new_all_same(Au::from_px(1));

        let border_display_item = ~BorderDisplayItem {
            base: BaseDisplayItem {
                bounds: absolute_box_bounds,
                node: self.node,
            },
            border: debug_border,
            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::solid)

        };
        stacking_context.content.push(BorderDisplayItemClass(border_display_item));

        // Draw a rectangle representing the baselines.
        let ascent = text_box.run.metrics_for_range(&text_box.range).ascent;
        let baseline = Rect(absolute_box_bounds.origin + Point2D(Au(0), ascent),
                            Size2D(absolute_box_bounds.size.width, Au(0)));

        let line_display_item = ~LineDisplayItem {
            base: BaseDisplayItem {
                bounds: baseline,
                node: self.node,
            },
            color: rgb(0, 200, 0),
            style: border_style::dashed,

        };
        stacking_context.content.push(LineDisplayItemClass(line_display_item))
    }

    fn build_debug_borders_around_box(&self,
                                      stacking_context: &mut StackingContext,
                                      flow_origin: Point2D<Au>) {
        let box_bounds = self.border_box.borrow();
        let absolute_box_bounds = box_bounds.translate(&flow_origin);

        // This prints a debug border around the border of this box.
        let debug_border = SideOffsets2D::new_all_same(Au::from_px(1));

        let border_display_item = ~BorderDisplayItem {
            base: BaseDisplayItem {
                bounds: absolute_box_bounds,
                node: self.node,
            },
            border: debug_border,
            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::solid)

        };
        stacking_context.content.push(BorderDisplayItemClass(border_display_item))
    }

    /// Adds the display items for this box to the given stacking context.
    ///
    /// Arguments:
    ///
    /// * `stacking_context`: The stacking context to add display items to.
    /// * `builder`: The display list builder, which manages the coordinate system and options.
    /// * `dirty`: The dirty rectangle in the coordinate system of the owning flow.
    /// * `flow_origin`: Position of the origin of the owning flow wrt the display list root flow.
    ///   box.
    /// * `flow`: The flow that this box belongs to.
    pub fn build_display_list(&self,
                              stacking_context: &mut StackingContext,
                              builder: &DisplayListBuilder,
                              _: &DisplayListBuildingInfo,
                              flow_origin: Point2D<Au>,
                              flow: &Flow,
                              background_and_border_level: BackgroundAndBorderLevel) {
        // Box position wrt to the owning flow.
        let box_bounds = *self.border_box.borrow();
        let absolute_box_bounds = box_bounds.translate(&flow_origin);
        debug!("Box::build_display_list at rel={}, abs={}: {:s}",
               box_bounds,
               absolute_box_bounds,
               self.debug_str());
        debug!("Box::build_display_list: dirty={}, flow_origin={}", builder.dirty, flow_origin);

        if self.style().InheritedBox.get().visibility != visibility::visible {
            return
        }

        if !absolute_box_bounds.intersects(&builder.dirty) {
            debug!("Box::build_display_list: Did not intersect...");
            return
        }

        debug!("Box::build_display_list: intersected. Adding display item...");

        {
            let list =
                stacking_context.list_for_background_and_border_level(background_and_border_level);

            // Add a background to the list, if this is an inline.
            //
            // FIXME(pcwalton): This is kind of ugly; merge with the call below?
            self.paint_inline_background_border_if_applicable(list,
                                                              &absolute_box_bounds,
                                                              &flow_origin);

            // Add the background to the list, if applicable.
            self.paint_background_if_applicable(list, builder, &absolute_box_bounds);

            // Add a border, if applicable.
            //
            // TODO: Outlines.
            self.paint_borders_if_applicable(list, &absolute_box_bounds);
        }

        match self.specific {
            UnscannedTextBox(_) => fail!("Shouldn't see unscanned boxes here."),
            TableColumnBox(_) => fail!("Shouldn't see table column boxes here."),
            ScannedTextBox(ref text_box) => {
                let text_color = self.style().Color.get().color.to_gfx_color();

                // Set the various text display item flags.
                let mut flow_flags = flow::base(flow).flags_info.clone();

                let inline_info = self.inline_info.borrow();
                match &*inline_info {
                    &Some(ref info) => {
                        for data in info.parent_info.as_slice().rev_iter() {
                            let parent_info = FlowFlagsInfo::new(&*data.style);
                            flow_flags.propagate_text_decoration_from_parent(&parent_info);
                        }
                    },
                    &None => {}
                }
                let mut text_flags = TextDisplayItemFlags::new();
                text_flags.set_override_underline(flow_flags.flags.override_underline());
                text_flags.set_override_overline(flow_flags.flags.override_overline());
                text_flags.set_override_line_through(flow_flags.flags.override_line_through());

                let mut bounds = absolute_box_bounds.clone();
                bounds.origin.x = bounds.origin.x + self.noncontent_left()
                                  + self.noncontent_inline_left();
                bounds.size.width = bounds.size.width - self.noncontent_width()
                                    - self.noncontent_inline_left()
                                    - self.noncontent_inline_right();

                // Create the text box.
                let text_display_item = ~TextDisplayItem {
                    base: BaseDisplayItem {
                        bounds: bounds,
                        node: self.node,
                    },
                    text_run: text_box.run.clone(),
                    range: text_box.range,
                    text_color: text_color,
                    overline_color: flow_flags.overline_color(text_color),
                    underline_color: flow_flags.underline_color(text_color),
                    line_through_color: flow_flags.line_through_color(text_color),
                    flags: text_flags,
                };

                stacking_context.content.push(TextDisplayItemClass(text_display_item));

                // Draw debug frames for text bounds.
                //
                // FIXME(pcwalton): This is a bit of an abuse of the logging infrastructure. We
                // should have a real `SERVO_DEBUG` system.
                debug!("{:?}", self.build_debug_borders_around_text_boxes(stacking_context,
                                                                          flow_origin,
                                                                          text_box))
            },
            GenericBox | IframeBox(..) | TableBox | TableCellBox | TableRowBox |
            TableWrapperBox => {
                let item = ~ClipDisplayItem {
                    base: BaseDisplayItem {
                        bounds: absolute_box_bounds,
                        node: self.node,
                    },
                    child_list: SmallVec0::new(),
                    need_clip: self.needs_clip()
                };
                stacking_context.content.push(ClipDisplayItemClass(item));

                // FIXME(pcwalton): This is a bit of an abuse of the logging infrastructure. We
                // should have a real `SERVO_DEBUG` system.
                debug!("{:?}", self.build_debug_borders_around_box(stacking_context, flow_origin))
            },
            ImageBox(ref image_box) => {
                let mut image_ref = image_box.image.borrow_mut();
                let mut bounds = absolute_box_bounds.clone();
                bounds.origin.x = bounds.origin.x + self.noncontent_left()
                                  + self.noncontent_inline_left();
                bounds.origin.y = bounds.origin.y + self.noncontent_top();
                bounds.size.width = bounds.size.width
                                    - self.noncontent_width() - self.noncontent_inline_left()
                                    - self.noncontent_inline_right();
                bounds.size.height = bounds.size.height - self.noncontent_height();

                match image_ref.get_image() {
                    Some(image) => {
                        debug!("(building display list) building image box");

                        // Place the image into the display list.
                        let image_display_item = ~ImageDisplayItem {
                            base: BaseDisplayItem {
                                bounds: bounds,
                                node: self.node,
                            },
                            image: image.clone(),
                            stretch_size: bounds.size,
                        };
                        stacking_context.content.push(ImageDisplayItemClass(image_display_item))
                    }
                    None => {
                        // No image data at all? Do nothing.
                        //
                        // TODO: Add some kind of placeholder image.
                        debug!("(building display list) no image :(");
                    }
                }
                // FIXME(pcwalton): This is a bit of an abuse of the logging infrastructure. We
                // should have a real `SERVO_DEBUG` system.
                debug!("{:?}", self.build_debug_borders_around_box(stacking_context, flow_origin))
            }
        }

        // If this is an iframe, then send its position and size up to the constellation.
        //
        // FIXME(pcwalton): Doing this during display list construction seems potentially
        // problematic if iframes are outside the area we're computing the display list for, since
        // they won't be able to reflow at all until the user scrolls to them. Perhaps we should
        // separate this into two parts: first we should send the size only to the constellation
        // once that's computed during assign-heights, and second we should should send the origin
        // to the constellation here during display list construction. This should work because
        // layout for the iframe only needs to know size, and origin is only relevant if the
        // iframe is actually going to be displayed.
        match self.specific {
            IframeBox(ref iframe_box) => {
                self.finalize_position_and_size_of_iframe(iframe_box, flow_origin, builder.ctx)
            }
            _ => {}
        }
    }

    /// Returns the intrinsic widths of this fragment.
    pub fn intrinsic_widths(&self) -> IntrinsicWidths {
        let mut result = self.style_specified_intrinsic_width();

        match self.specific {
            GenericBox | IframeBox(_) | TableBox | TableCellBox | TableColumnBox(_) | TableRowBox |
            TableWrapperBox => {}
            ImageBox(ref image_box_info) => {
                let image_width = image_box_info.image_width();
                result.minimum_width = geometry::max(result.minimum_width, image_width);
                result.preferred_width = geometry::max(result.preferred_width, image_width);
            }
            ScannedTextBox(ref text_box_info) => {
                let range = &text_box_info.range;
                let min_line_width = text_box_info.run.min_width_for_range(range);

                let mut max_line_width = Au::new(0);
                for line_range in text_box_info.run.iter_natural_lines_for_range(range) {
                    let line_metrics = text_box_info.run.metrics_for_range(&line_range);
                    max_line_width = Au::max(max_line_width, line_metrics.advance_width);
                }

                result.minimum_width = geometry::max(result.minimum_width, min_line_width);
                result.preferred_width = geometry::max(result.preferred_width, max_line_width);
            }
            UnscannedTextBox(..) => fail!("Unscanned text boxes should have been scanned by now!"),
        }

        // Take borders and padding for parent inline boxes into account.
        match *self.inline_info.borrow() {
            None => {}
            Some(ref inline_info) => {
                for inline_parent_info in inline_info.parent_info.iter() {
                    let border_width = inline_parent_info.border.left +
                        inline_parent_info.border.right;
                    let padding_width = inline_parent_info.padding.left +
                        inline_parent_info.padding.right;
                    result.minimum_width = result.minimum_width + border_width + padding_width;
                    result.preferred_width = result.preferred_width + border_width + padding_width;
                }
            }
        }

        result
    }


    /// TODO: What exactly does this function return? Why is it Au(0) for GenericBox?
    pub fn content_width(&self) -> Au {
        match self.specific {
            GenericBox | IframeBox(_) | TableBox | TableCellBox | TableRowBox |
            TableWrapperBox => Au(0),
            ImageBox(ref image_box_info) => {
                image_box_info.computed_width()
            }
            ScannedTextBox(ref text_box_info) => {
                let (range, run) = (&text_box_info.range, &text_box_info.run);
                let text_bounds = run.metrics_for_range(range).bounding_box;
                text_bounds.size.width
            }
            TableColumnBox(_) => fail!("Table column boxes do not have width"),
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
        }
    }

    /// Returns, and computes, the height of this box.
    pub fn content_height(&self) -> Au {
        match self.specific {
            GenericBox | IframeBox(_) | TableBox | TableCellBox | TableRowBox |
            TableWrapperBox => Au(0),
            ImageBox(ref image_box_info) => {
                image_box_info.computed_height()
            }
            ScannedTextBox(ref text_box_info) => {
                // Compute the height based on the line-height and font size.
                let (range, run) = (&text_box_info.range, &text_box_info.run);
                let text_bounds = run.metrics_for_range(range).bounding_box;
                let em_size = text_bounds.size.height;
                self.calculate_line_height(em_size)
            }
            TableColumnBox(_) => fail!("Table column boxes do not have height"),
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
        }
    }

    /// Return the size of the content box.
    pub fn content_box_size(&self) -> Size2D<Au> {
        let border_box_size = self.border_box.borrow().size;
        Size2D(border_box_size.width - self.noncontent_width(),
               border_box_size.height - self.noncontent_height())
    }

    /// Split box which includes new-line character
    pub fn split_by_new_line(&self) -> SplitBoxResult {
        match self.specific {
            GenericBox | IframeBox(_) | ImageBox(_) | TableBox | TableCellBox |
            TableRowBox | TableWrapperBox => CannotSplit,
            TableColumnBox(_) => fail!("Table column boxes do not need to split"),
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
            ScannedTextBox(ref text_box_info) => {
                let mut new_line_pos = self.new_line_pos.clone();
                let cur_new_line_pos = new_line_pos.shift().unwrap();

                let left_range = Range::new(text_box_info.range.begin(), cur_new_line_pos);
                let right_range = Range::new(text_box_info.range.begin() + cur_new_line_pos + 1, text_box_info.range.length() - (cur_new_line_pos + 1));

                // Left box is for left text of first founded new-line character.
                let left_box = {
                    let new_text_box_info = ScannedTextBoxInfo::new(text_box_info.run.clone(), left_range);
                    let new_metrics = new_text_box_info.run.metrics_for_range(&left_range);
                    let mut new_box = self.transform(new_metrics.bounding_box.size, ScannedTextBox(new_text_box_info));
                    new_box.new_line_pos = ~[];
                    Some(new_box)
                };

                // Right box is for right text of first founded new-line character.
                let right_box = if right_range.length() > 0 {
                    let new_text_box_info = ScannedTextBoxInfo::new(text_box_info.run.clone(), right_range);
                    let new_metrics = new_text_box_info.run.metrics_for_range(&right_range);
                    let mut new_box = self.transform(new_metrics.bounding_box.size, ScannedTextBox(new_text_box_info));
                    new_box.new_line_pos = new_line_pos;
                    Some(new_box)
                } else {
                    None
                };

                SplitDidFit(left_box, right_box)
            }
        }
    }

    /// Attempts to split this box so that its width is no more than `max_width`.
    pub fn split_to_width(&self, max_width: Au, starts_line: bool) -> SplitBoxResult {
        match self.specific {
            GenericBox | IframeBox(_) | ImageBox(_) | TableBox | TableCellBox |
            TableRowBox | TableWrapperBox => CannotSplit,
            TableColumnBox(_) => fail!("Table column boxes do not have width"),
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
            ScannedTextBox(ref text_box_info) => {
                let mut pieces_processed_count: uint = 0;
                let mut remaining_width: Au = max_width;
                let mut left_range = Range::new(text_box_info.range.begin(), 0);
                let mut right_range: Option<Range> = None;

                debug!("split_to_width: splitting text box (strlen={:u}, range={}, \
                                                            avail_width={})",
                       text_box_info.run.text.len(),
                       text_box_info.range,
                       max_width);

                for (glyphs, offset, slice_range) in text_box_info.run.iter_slices_for_range(
                        &text_box_info.range) {
                    debug!("split_to_width: considering slice (offset={}, range={}, \
                                                               remain_width={})",
                           offset,
                           slice_range,
                           remaining_width);

                    let metrics = text_box_info.run.metrics_for_slice(glyphs, &slice_range);
                    let advance = metrics.advance_width;

                    let should_continue;
                    if advance <= remaining_width {
                        should_continue = true;

                        if starts_line && pieces_processed_count == 0 && glyphs.is_whitespace() {
                            debug!("split_to_width: case=skipping leading trimmable whitespace");
                            left_range.shift_by(slice_range.length() as int);
                        } else {
                            debug!("split_to_width: case=enlarging span");
                            remaining_width = remaining_width - advance;
                            left_range.extend_by(slice_range.length() as int);
                        }
                    } else {
                        // The advance is more than the remaining width.
                        should_continue = false;
                        let slice_begin = offset + slice_range.begin();
                        let slice_end = offset + slice_range.end();

                        if glyphs.is_whitespace() {
                            // If there are still things after the trimmable whitespace, create the
                            // right chunk.
                            if slice_end < text_box_info.range.end() {
                                debug!("split_to_width: case=skipping trimmable trailing \
                                        whitespace, then split remainder");
                                let right_range_end = text_box_info.range.end() - slice_end;
                                right_range = Some(Range::new(slice_end, right_range_end));
                            } else {
                                debug!("split_to_width: case=skipping trimmable trailing \
                                        whitespace");
                            }
                        } else if slice_begin < text_box_info.range.end() {
                            // There are still some things left over at the end of the line. Create
                            // the right chunk.
                            let right_range_end = text_box_info.range.end() - slice_begin;
                            right_range = Some(Range::new(slice_begin, right_range_end));
                            debug!("split_to_width: case=splitting remainder with right range={:?}",
                                   right_range);
                        }
                    }

                    pieces_processed_count += 1;

                    if !should_continue {
                        break
                    }
                }

                let left_box = if left_range.length() > 0 {
                    let new_text_box_info = ScannedTextBoxInfo::new(text_box_info.run.clone(), left_range);
                    let mut new_metrics = new_text_box_info.run.metrics_for_range(&left_range);
                    new_metrics.bounding_box.size.height = self.border_box.borrow().size.height;
                    Some(self.transform(new_metrics.bounding_box.size,
                                        ScannedTextBox(new_text_box_info)))
                } else {
                    None
                };

                let right_box = right_range.map_or(None, |range: Range| {
                    let new_text_box_info = ScannedTextBoxInfo::new(text_box_info.run.clone(), range);
                    let mut new_metrics = new_text_box_info.run.metrics_for_range(&range);
                    new_metrics.bounding_box.size.height = self.border_box.borrow().size.height;
                    Some(self.transform(new_metrics.bounding_box.size,
                                        ScannedTextBox(new_text_box_info)))
                });

                if pieces_processed_count == 1 || left_box.is_none() {
                    SplitDidNotFit(left_box, right_box)
                } else {
                    if left_box.is_some() {
                        left_box.get_ref().clear_noncontent_inline_right();
                    }
                    if right_box.is_some() {
                        right_box.get_ref().clear_noncontent_inline_left();
                    }
                    SplitDidFit(left_box, right_box)
                }
            }
        }
    }

    /// Returns true if this box is an unscanned text box that consists entirely of whitespace.
    pub fn is_whitespace_only(&self) -> bool {
        match self.specific {
            UnscannedTextBox(ref text_box_info) => is_whitespace(text_box_info.text),
            _ => false,
        }
    }

    /// Assigns replaced width, padding, and margins for this box only if it is replaced content
    /// per CSS 2.1 § 10.3.2.
    pub fn assign_replaced_width_if_necessary(&self, container_width: Au) {
        match self.specific {
            GenericBox | IframeBox(_) | TableBox | TableCellBox | TableRowBox |
            TableWrapperBox => {}
            ImageBox(ref image_box_info) => {
                self.compute_padding(self.style(), container_width);

                // TODO(ksh8281): compute border,margin
                let width = ImageBoxInfo::style_length(self.style().Box.get().width,
                                                       image_box_info.dom_width,
                                                       container_width);

                // FIXME(ksh8281): we shouldn't figure height this way
                // now, we don't know about size of parent's height
                let height = ImageBoxInfo::style_length(self.style().Box.get().height,
                                                       image_box_info.dom_height,
                                                       Au::new(0));

                let width = match (width,height) {
                    (Auto,Auto) => {
                        image_box_info.image_width()
                    },
                    (Auto,Specified(h)) => {
                        let scale = image_box_info.
                            image_height().to_f32().unwrap() / h.to_f32().unwrap();
                        Au::new((image_box_info.image_width().to_f32().unwrap() / scale) as i32)
                    },
                    (Specified(w),_) => {
                        w
                    }
                };

                let mut position = self.border_box.borrow_mut();
                position.size.width = width + self.noncontent_width() +
                    self.noncontent_inline_left() + self.noncontent_inline_right();
                *image_box_info.computed_width.borrow_mut() = Some(width);
            }
            ScannedTextBox(_) => {
                // Scanned text boxes will have already had their
                // content_widths assigned by this point.
                let mut position = self.border_box.borrow_mut();
                position.size.width = position.size.width + self.noncontent_width() +
                    self.noncontent_inline_left() + self.noncontent_inline_right();
            }
            TableColumnBox(_) => fail!("Table column boxes do not have width"),
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
        }
    }

    /// Assign height for this box if it is replaced content.
    ///
    /// Ideally, this should follow CSS 2.1 § 10.6.2
    pub fn assign_replaced_height_if_necessary(&self) {
        match self.specific {
            GenericBox | IframeBox(_) | TableBox | TableCellBox | TableRowBox |
            TableWrapperBox => {}
            ImageBox(ref image_box_info) => {
                // TODO(ksh8281): compute border,margin,padding
                let width = image_box_info.computed_width();
                // FIXME(ksh8281): we shouldn't assign height this way
                // we don't know about size of parent's height
                let height = ImageBoxInfo::style_length(self.style().Box.get().height,
                                                        image_box_info.dom_height,
                                                        Au::new(0));

                let height = match (self.style().Box.get().width,
                                    image_box_info.dom_width,
                                    height) {
                    (LPA_Auto, None, Auto) => {
                        image_box_info.image_height()
                    },
                    (_,_,Auto) => {
                        let scale = image_box_info.image_width().to_f32().unwrap()
                            / width.to_f32().unwrap();
                        Au::new((image_box_info.image_height().to_f32().unwrap() / scale) as i32)
                    },
                    (_,_,Specified(h)) => {
                        h
                    }
                };

                let mut position = self.border_box.borrow_mut();
                *image_box_info.computed_height.borrow_mut() = Some(height);
                position.size.height = height + self.noncontent_height()
            }
            ScannedTextBox(_) => {
                // Scanned text boxes will have already had their widths assigned by this point
                let mut position = self.border_box.borrow_mut();
                // Scanned text boxes' content heights are calculated by the
                // text run scanner during Flow construction.
                position.size.height
                    = position.size.height + self.noncontent_height()
            }
            TableColumnBox(_) => fail!("Table column boxes do not have height"),
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
        }
    }

    /// Returns true if this box can merge with another adjacent box or false otherwise.
    pub fn can_merge_with_box(&self, other: &Box) -> bool {
        match (&self.specific, &other.specific) {
            (&UnscannedTextBox(_), &UnscannedTextBox(_)) => {
                self.font_style() == other.font_style() &&
                    self.text_decoration() == other.text_decoration()
            }
            _ => false,
        }
    }

    /// Cleans up all the memory associated with this box.
    pub fn teardown(&self) {
        match self.specific {
            ScannedTextBox(ref text_box_info) => text_box_info.run.teardown(),
            _ => {}
        }
    }

    /// Returns true if the contents should be clipped (i.e. if `overflow` is `hidden`).
    pub fn needs_clip(&self) -> bool {
        self.style().Box.get().overflow == overflow::hidden
    }

    /// Returns a debugging string describing this box.
    pub fn debug_str(&self) -> ~str {
        let class_name = match self.specific {
            GenericBox => "GenericBox",
            IframeBox(_) => "IframeBox",
            ImageBox(_) => "ImageBox",
            ScannedTextBox(_) => "ScannedTextBox",
            TableBox => "TableBox",
            TableCellBox => "TableCellBox",
            TableColumnBox(_) => "TableColumnBox",
            TableRowBox => "TableRowBox",
            TableWrapperBox => "TableWrapperBox",
            UnscannedTextBox(_) => "UnscannedTextBox",
        };

        format!("({}{}{}{})",
                class_name,
                self.side_offsets_debug_string("b", *self.border.borrow()),
                self.side_offsets_debug_string("p", *self.padding.borrow()),
                self.side_offsets_debug_string("m", *self.margin.borrow()))
    }

    /// A helper function to return a debug string describing the side offsets for one of the rect
    /// box model properties (border, padding, or margin).
    fn side_offsets_debug_string(&self, name: &str, value: SideOffsets2D<Au>) -> ~str {
        let zero: SideOffsets2D<Au> = Zero::zero();
        if value == zero {
            return "".to_str()
        }
        format!(" {}{},{},{},{}",
                name,
                value.top,
                value.right,
                value.bottom,
                value.left)
    }

    /// Sends the size and position of this iframe box to the constellation. This is out of line to
    /// guide inlining.
    #[inline(never)]
    fn finalize_position_and_size_of_iframe(&self,
                                            iframe_box: &IframeBoxInfo,
                                            offset: Point2D<Au>,
                                            layout_context: &LayoutContext) {
        let left = offset.x + self.margin.borrow().left + self.border.borrow().left +
            self.padding.borrow().left;
        let top = offset.y + self.margin.borrow().top + self.border.borrow().top +
            self.padding.borrow().top;
        let width = self.border_box.borrow().size.width - self.noncontent_width();
        let height = self.border_box.borrow().size.height - self.noncontent_height();
        let origin = Point2D(geometry::to_frac_px(left) as f32, geometry::to_frac_px(top) as f32);
        let size = Size2D(geometry::to_frac_px(width) as f32, geometry::to_frac_px(height) as f32);
        let rect = Rect(origin, size);

        debug!("finalizing position and size of iframe for {:?},{:?}",
               iframe_box.pipeline_id,
               iframe_box.subpage_id);
        let msg = FrameRectMsg(iframe_box.pipeline_id, iframe_box.subpage_id, rect);
        let ConstellationChan(ref chan) = layout_context.constellation_chan;
        chan.send(msg)
    }
}
