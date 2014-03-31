/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Box` type, which represents the leaves of the layout tree.

use extra::url::Url;
use sync::{MutexArc, Arc};
use geom::{Point2D, Rect, Size2D, SideOffsets2D};
use geom::approxeq::ApproxEq;
use gfx::color::rgb;
use gfx::display_list::{BaseDisplayItem, BorderDisplayItem, BorderDisplayItemClass};
use gfx::display_list::{LineDisplayItem, LineDisplayItemClass};
use gfx::display_list::{ImageDisplayItem, ImageDisplayItemClass};
use gfx::display_list::{SolidColorDisplayItem, SolidColorDisplayItemClass, TextDisplayItem};
use gfx::display_list::{TextDisplayItemClass, TextDisplayItemFlags, ClipDisplayItem};
use gfx::display_list::{ClipDisplayItemClass, DisplayListCollection};
use gfx::font::FontStyle;
use gfx::text::text_run::TextRun;
use servo_msg::constellation_msg::{ConstellationChan, FrameRectMsg, PipelineId, SubpageId};
use servo_net::image::holder::ImageHolder;
use servo_net::local_image_cache::LocalImageCache;
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::range::*;
use servo_util::namespace;
use servo_util::str::is_whitespace;

use std::cast;
use std::cell::RefCell;
use std::num::Zero;
use style::{ComputedValues, TElement, TNode, cascade, initial_values};
use style::computed_values::{LengthOrPercentage, LengthOrPercentageOrAuto, overflow, LPA_Auto};
use style::computed_values::{border_style, clear, font_family, line_height, position};
use style::computed_values::{text_align, text_decoration, vertical_align, visibility, white_space};

use css::node_style::StyledNode;
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData, ToGfxColor};
use layout::floats::{ClearBoth, ClearLeft, ClearRight, ClearType};
use layout::flow::{Flow, FlowFlagsInfo};
use layout::flow;
use layout::model::{MaybeAuto, specified, Auto, Specified};
use layout::util::OpaqueNode;
use layout::wrapper::{TLayoutNode, ThreadSafeLayoutNode};

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
    node: OpaqueNode,

    /// The CSS style of this box.
    style: Arc<ComputedValues>,

    /// The position of this box relative to its owning flow.
    /// The size includes padding and border, but not margin.
    border_box: RefCell<Rect<Au>>,

    /// The border of the content box.
    ///
    /// FIXME(pcwalton): This need not be stored in the box.
    border: RefCell<SideOffsets2D<Au>>,

    /// The padding of the content box.
    padding: RefCell<SideOffsets2D<Au>>,

    /// The margin of the content box.
    margin: RefCell<SideOffsets2D<Au>>,

    /// Info specific to the kind of box. Keep this enum small.
    specific: SpecificBoxInfo,

    /// positioned box offsets
    position_offsets: RefCell<SideOffsets2D<Au>>,

    /// Inline data
    inline_info: RefCell<Option<InlineInfo>>,

    /// New-line chracter(\n)'s positions(relative, not absolute)
    new_line_pos: ~[uint],
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
    image: RefCell<ImageHolder>,
    computed_width: RefCell<Option<Au>>,
    computed_height: RefCell<Option<Au>>,
    dom_width: Option<Au>,
    dom_height: Option<Au>,
}

impl ImageBoxInfo {
    /// Creates a new image box from the given URL and local image cache.
    ///
    /// FIXME(pcwalton): The fact that image boxes store the cache in the box makes little sense to
    /// me.
    pub fn new(node: &ThreadSafeLayoutNode,
               image_url: Url,
               local_image_cache: MutexArc<LocalImageCache>)
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
        match self.computed_width.borrow().get() {
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
        Au::from_px(image_ref.get().get_size().unwrap_or(Size2D(0,0)).width)
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
        match self.computed_height.borrow().get() {
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
        Au::from_px(image_ref.get().get_size().unwrap_or(Size2D(0,0)).height)
    }
}

/// A box that represents an inline frame (iframe). This stores the pipeline ID so that the size
/// of this iframe can be communicated via the constellation to the iframe's own layout task.
#[deriving(Clone)]
pub struct IframeBoxInfo {
    /// The pipeline ID of this iframe.
    pipeline_id: PipelineId,
    /// The subpage ID of this iframe.
    subpage_id: SubpageId,
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
    run: Arc<~TextRun>,

    /// The range within the above text run that this represents.
    range: Range,
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
    text: ~str,
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


/// data for inline boxes
#[deriving(Clone)]
pub struct InlineInfo {
    parent_info: ~[InlineParentInfo],
    baseline: Au,
}

impl InlineInfo {
    pub fn new() -> InlineInfo {
        InlineInfo {
            parent_info: ~[],
            baseline: Au::new(0),
        }
    }
}

#[deriving(Clone)]
pub struct InlineParentInfo {
    padding: SideOffsets2D<Au>,
    border: SideOffsets2D<Au>,
    margin: SideOffsets2D<Au>,
    style: Arc<ComputedValues>,
    font_ascent: Au,
    font_descent: Au,
    node: OpaqueNode,
}

/// A box that represents a table column.
#[deriving(Clone)]
pub struct TableColumnBoxInfo {
    /// the number of columns a <col> element should span
    span: Option<int>,
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
            self.border.get().$side + self.padding.get().$side
        }

        pub fn $inline_get(&self) -> Au {
            let mut val = Au::new(0);
            let info = self.inline_info.borrow();
            match info.get() {
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

            match other_info.get() {
                &Some(ref other_info) => {
                    match info.get() {
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
            match info.get() {
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
    /// Constructs a new `Box` instance.
    pub fn new(constructor: &mut FlowConstructor, node: &ThreadSafeLayoutNode) -> Box {
        Box {
            node: OpaqueNode::from_thread_safe_layout_node(node),
            style: node.style().clone(),
            border_box: RefCell::new(Au::zero_rect()),
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
            node: OpaqueNode::from_thread_safe_layout_node(node),
            style: node.style().clone(),
            border_box: RefCell::new(Au::zero_rect()),
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

        let (node_style, _) = cascade(&[], false, Some(node.style().get()),
                                      &initial_values(), None);
        Box {
            node: OpaqueNode::from_thread_safe_layout_node(node),
            style: Arc::new(node_style),
            border_box: RefCell::new(Au::zero_rect()),
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
            border_box: RefCell::new(Au::zero_rect()),
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

    // CSS Section 10.6.4
    // We have to solve the constraint equation:
    // top + bottom + height + (vertical border + padding) = height of
    // containing block (`screen_height`)
    //
    // `y`: static position of the element
    //TODO(ibnc) take into account padding.
    pub fn get_y_coord_and_new_height_if_fixed(&self,
                                               screen_height: Au,
                                               mut height: Au,
                                               mut y: Au,
                                               is_fixed: bool)
                                               -> (Au, Au) {
        if is_fixed {
            let position_offsets = self.position_offsets.get();
            match (position_offsets.top, position_offsets.bottom) {
                (Au(0), Au(0)) => {}
                (Au(0), _) => {
                    y = screen_height - position_offsets.bottom - height;
                }
                (_, Au(0)) => {
                    y = position_offsets.top;
                }
                (_, _) => {
                    y = position_offsets.top;
                    match MaybeAuto::from_style(self.style().Box.get().height, Au(0)) {
                        Auto => {
                            height = screen_height - position_offsets.top - position_offsets.bottom;
                        }
                        _ => {}
                    }
                }
            }
        }
        return (y, height);
    }

    // CSS Section 10.3.7
    //TODO(ibnc) removing padding when width needs to be stretched.
    pub fn get_x_coord_and_new_width_if_fixed(&self,
                                              screen_width: Au,
                                              screen_height: Au,
                                              mut width: Au,
                                              mut x: Au,
                                              is_fixed: bool)
                                              -> (Au, Au) {
        if is_fixed {
            self.compute_positioned_offsets(self.style(), screen_width, screen_height);
            let position_offsets = self.position_offsets.get();

            match (position_offsets.left, position_offsets.right) {
                (Au(0), Au(0)) => {}
                (_, Au(0)) => {
                   x = position_offsets.left;
                }
                (Au(0), _) => {
                    x = screen_width - position_offsets.right - width;
                }
                (_, _) => {
                    x = position_offsets.left;
                    match MaybeAuto::from_style(self.style().Box.get().width, Au(0)) {
                        Auto => {
                            width = screen_width - position_offsets.left - position_offsets.right;
                        }
                        _ => {}
                    }
                }
            }
        }
        return (x, width);
    }

    /// Transforms this box into another box of the given type, with the given size, preserving all
    /// the other data.
    pub fn transform(&self, size: Size2D<Au>, specific: SpecificBoxInfo) -> Box {
        Box {
            node: self.node,
            style: self.style.clone(),
            border_box: RefCell::new(Rect(self.border_box.get().origin, size)),
            border: RefCell::new(self.border.get()),
            padding: RefCell::new(self.padding.get()),
            margin: RefCell::new(self.margin.get()),
            specific: specific,
            position_offsets: RefCell::new(Zero::zero()),
            inline_info: self.inline_info.clone(),
            new_line_pos: self.new_line_pos.clone(),
        }
    }

    /// Returns the shared part of the width for computation of minimum and preferred width per
    /// CSS 2.1.
    fn guess_width(&self) -> Au {
        let style = self.style();
        let mut margin_left = Au::new(0);
        let mut margin_right = Au::new(0);
        let mut padding_left = Au::new(0);
        let mut padding_right = Au::new(0);

        match self.specific {
            GenericBox | IframeBox(_) | ImageBox(_) => {
                margin_left = MaybeAuto::from_style(style.Margin.get().margin_left,
                                                    Au::new(0)).specified_or_zero();
                margin_right = MaybeAuto::from_style(style.Margin.get().margin_right,
                                                     Au::new(0)).specified_or_zero();
                padding_left = self.compute_padding_length(style.Padding.get().padding_left,
                                                           Au::new(0));
                padding_right = self.compute_padding_length(style.Padding.get().padding_right,
                                                            Au::new(0));
            }
            TableBox | TableCellBox => {
                padding_left = self.compute_padding_length(style.Padding.get().padding_left,
                                                           Au::new(0));
                padding_right = self.compute_padding_length(style.Padding.get().padding_right,
                                                            Au::new(0));
            }
            TableWrapperBox => {
                margin_left = MaybeAuto::from_style(style.Margin.get().margin_left,
                                                    Au::new(0)).specified_or_zero();
                margin_right = MaybeAuto::from_style(style.Margin.get().margin_right,
                                                     Au::new(0)).specified_or_zero();
            }
            TableRowBox => {}
            ScannedTextBox(_) | TableColumnBox(_) | UnscannedTextBox(_) => return Au(0),
        }

        let width = MaybeAuto::from_style(style.Box.get().width, Au::new(0)).specified_or_zero();

        width + margin_left + margin_right + padding_left + padding_right +
            self.border.get().left + self.border.get().right
    }

    pub fn calculate_line_height(&self, font_size: Au) -> Au {
        match self.line_height() {
            line_height::Normal => font_size.scale_by(1.14),
            line_height::Number(l) => font_size.scale_by(l),
            line_height::Length(l) => l
        }
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
        self.border.set(border)
    }

    pub fn compute_positioned_offsets(&self, style: &ComputedValues, containing_width: Au, containing_height: Au) {
        self.position_offsets.set(SideOffsets2D::new(
                MaybeAuto::from_style(style.PositionOffsets.get().top, containing_height)
                .specified_or_zero(),
                MaybeAuto::from_style(style.PositionOffsets.get().right, containing_width)
                .specified_or_zero(),
                MaybeAuto::from_style(style.PositionOffsets.get().bottom, containing_height)
                .specified_or_zero(),
                MaybeAuto::from_style(style.PositionOffsets.get().left, containing_width)
                .specified_or_zero()));
    }

    /// Compute and set margin-top and margin-bottom values.
    ///
    /// If a value is specified or is a percentage, we calculate the right value here.
    /// If it is auto, it is up to assign-height to ignore this value and
    /// calculate the correct margin values.
    pub fn compute_margin_top_bottom(&self, containing_block_width: Au) {
        match self.specific {
            TableBox | TableCellBox | TableRowBox | TableColumnBox(_) => {
                self.margin.set(SideOffsets2D::new(Au(0), Au(0), Au(0), Au(0)))
            },
            _ => {
                let style = self.style();
                // Note: CSS 2.1 defines margin % values wrt CB *width* (not height).
                let margin_top = MaybeAuto::from_style(style.Margin.get().margin_top,
                                                       containing_block_width).specified_or_zero();
                let margin_bottom = MaybeAuto::from_style(style.Margin.get().margin_bottom,
                                                          containing_block_width).specified_or_zero();
                let mut margin = self.margin.get();
                margin.top = margin_top;
                margin.bottom = margin_bottom;
                self.margin.set(margin);
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
        self.padding.set(padding)
    }

    fn compute_padding_length(&self, padding: LengthOrPercentage, content_box_width: Au) -> Au {
        specified(padding, content_box_width)
    }

    pub fn padding_box_size(&self) -> Size2D<Au> {
        let border_box_size = self.border_box.get().size;
        Size2D(border_box_size.width - self.border.get().left - self.border.get().right,
               border_box_size.height - self.border.get().top - self.border.get().bottom)
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
        match info.get() {
            &Some(ref info) => {
                for info in info.parent_info.iter() {
                    if info.style.get().Box.get().position == position::relative {
                        rel_pos.x = rel_pos.x + left_right(info.style.get(),
                                                           container_block_size.width);
                        rel_pos.y = rel_pos.y + top_bottom(info.style.get(),
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
        let font_families = my_style.Font.get().font_family.map(|family| {
            match *family {
                font_family::FamilyName(ref name) => (*name).clone(),
            }
        });
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
        self.style.get()
    }

    /// Returns the text alignment of the computed style of the nearest ancestor-or-self `Element`
    /// node.
    pub fn text_align(&self) -> text_align::T {
        self.style().InheritedText.get().text_align
    }

    pub fn line_height(&self) -> line_height::T {
        self.style().InheritedBox.get().line_height
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
            TableWrapperBox => self.margin.get().left,
            TableBox | TableCellBox => self.border.get().left + self.padding.get().left,
            TableRowBox => self.border.get().left,
            TableColumnBox(_) => Au(0),
            _ => self.margin.get().left + self.border.get().left + self.padding.get().left
        }
    }

    /// Returns the top offset from margin edge to content edge.
    pub fn top_offset(&self) -> Au {
        match self.specific {
            TableWrapperBox => self.margin.get().top,
            TableBox | TableCellBox => self.border.get().top + self.padding.get().top,
            TableRowBox => self.border.get().top,
            TableColumnBox(_) => Au(0),
            _ => self.margin.get().top + self.border.get().top + self.padding.get().top
        }
    }

    /// Returns the bottom offset from margin edge to content edge.
    pub fn bottom_offset(&self) -> Au {
        match self.specific {
            TableWrapperBox => self.margin.get().bottom,
            TableBox | TableCellBox => self.border.get().bottom + self.padding.get().bottom,
            TableRowBox => self.border.get().bottom,
            TableColumnBox(_) => Au(0),
            _ => self.margin.get().bottom + self.border.get().bottom + self.padding.get().bottom
        }
    }

    /// Returns true if this element is replaced content. This is true for images, form elements,
    /// and so on.
    pub fn is_replaced(&self) -> bool {
        match self.specific {
            ImageBox(..) => true,
            _ => false,
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

    pub fn paint_inline_background_border_if_applicable<E:ExtraDisplayListData>(
                                          &self,
                                          index: uint,
                                          lists: &RefCell<DisplayListCollection<E>>,
                                          absolute_bounds: &Rect<Au>,
                                          offset: &Point2D<Au>) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a box".
        let info = self.inline_info.borrow();
        match info.get() {
            &Some(ref box_info) => {
                let mut bg_rect = absolute_bounds.clone();
                for info in box_info.parent_info.rev_iter() {
                    // TODO (ksh8281) compute vertical-align, line-height
                    bg_rect.origin.y = box_info.baseline + offset.y - info.font_ascent;
                    bg_rect.size.height = info.font_ascent + info.font_descent;
                    let background_color = info.style.get().resolve_color(
                        info.style.get().Background.get().background_color);

                    if !background_color.alpha.approx_eq(&0.0) {
                        lists.with_mut(|lists| {
                            let solid_color_display_item = ~SolidColorDisplayItem {
                                base: BaseDisplayItem {
                                          bounds: bg_rect.clone(),
                                          extra: ExtraDisplayListData::new(self),
                                      },
                                      color: background_color.to_gfx_color(),
                            };

                            lists.lists[index].append_item(SolidColorDisplayItemClass(solid_color_display_item))
                        });
                    }
                    let border = &info.border;
                    // Fast path.
                    if border.is_zero() {
                        continue;
                    }
                    bg_rect.origin.y = bg_rect.origin.y - border.top;
                    bg_rect.size.height = bg_rect.size.height + border.top + border.bottom;

                    let style = info.style.get();
                    let top_color = style.resolve_color(style.Border.get().border_top_color);
                    let right_color = style.resolve_color(style.Border.get().border_right_color);
                    let bottom_color = style.resolve_color(style.Border.get().border_bottom_color);
                    let left_color = style.resolve_color(style.Border.get().border_left_color);
                    let top_style = style.Border.get().border_top_style;
                    let right_style = style.Border.get().border_right_style;
                    let bottom_style = style.Border.get().border_bottom_style;
                    let left_style = style.Border.get().border_left_style;


                    lists.with_mut(|lists| {
                        let border_display_item = ~BorderDisplayItem {
                            base: BaseDisplayItem {
                                      bounds: bg_rect,
                                      extra: ExtraDisplayListData::new(self),
                                  },
                                  border: border.clone(),
                                  color: SideOffsets2D::new(top_color.to_gfx_color(),
                                  right_color.to_gfx_color(),
                                  bottom_color.to_gfx_color(),
                                  left_color.to_gfx_color()),
                                  style: SideOffsets2D::new(top_style,
                                  right_style,
                                  bottom_style,
                                  left_style)
                        };

                        lists.lists[index].append_item(BorderDisplayItemClass(border_display_item))
                    });

                    bg_rect.origin.x = bg_rect.origin.x + border.left;
                    bg_rect.size.width = bg_rect.size.width - border.left - border.right;
                }
            },
            &None => {}
        }
    }
    /// Adds the display items necessary to paint the background of this box to the display list if
    /// necessary.
    pub fn paint_background_if_applicable<E:ExtraDisplayListData>(
                                          &self,
                                          builder: &DisplayListBuilder,
                                          index: uint,
                                          lists: &RefCell<DisplayListCollection<E>>,
                                          absolute_bounds: &Rect<Au>) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a box".
        let style = self.style();
        let background_color = style.resolve_color(style.Background.get().background_color);
        if !background_color.alpha.approx_eq(&0.0) {
            lists.with_mut(|lists| {
                let solid_color_display_item = ~SolidColorDisplayItem {
                    base: BaseDisplayItem {
                        bounds: *absolute_bounds,
                        extra: ExtraDisplayListData::new(self),
                    },
                    color: background_color.to_gfx_color(),
                };

                lists.lists[index].append_item(SolidColorDisplayItemClass(solid_color_display_item))
            });
        }

        // The background image is painted on top of the background color.
        // Implements background image, per spec:
        // http://www.w3.org/TR/CSS21/colors.html#background
        match style.Background.get().background_image {
            Some(ref image_url) => {
                let mut holder = ImageHolder::new(image_url.clone(), builder.ctx.image_cache.clone());
                match holder.get_image() {
                    Some(image) => {
                        debug!("(building display list) building background image");

                        // Place the image into the display list.
                        lists.with_mut(|lists| {
                            let image_display_item = ~ImageDisplayItem {
                                base: BaseDisplayItem {
                                    bounds: *absolute_bounds,
                                    extra: ExtraDisplayListData::new(self),
                                },
                                image: image.clone(),
                            };
                            lists.lists[index].append_item(ImageDisplayItemClass(image_display_item));
                        });
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
    pub fn paint_borders_if_applicable<E:ExtraDisplayListData>(
                                       &self,
                                       index: uint,
                                       lists: &RefCell<DisplayListCollection<E>>,
                                       abs_bounds: &Rect<Au>) {
        // Fast path.
        let border = self.border.get();
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
        lists.with_mut(|lists| {
            let border_display_item = ~BorderDisplayItem {
                base: BaseDisplayItem {
                    bounds: abs_bounds,
                    extra: ExtraDisplayListData::new(self),
                },
                border: border,
                color: SideOffsets2D::new(top_color.to_gfx_color(),
                                          right_color.to_gfx_color(),
                                          bottom_color.to_gfx_color(),
                                          left_color.to_gfx_color()),
                style: SideOffsets2D::new(top_style,
                                          right_style,
                                          bottom_style,
                                          left_style)
            };

            lists.lists[index].append_item(BorderDisplayItemClass(border_display_item))
        });
    }

    /// Adds the display items for this box to the given display list.
    ///
    /// Arguments:
    /// * `builder`: The display list builder, which manages the coordinate system and options.
    /// * `dirty`: The dirty rectangle in the coordinate system of the owning flow.
    /// * `flow_origin`: Position of the origin of the owning flow wrt the display list root flow.
    ///   box.
    /// * `list`: The display list to which items should be appended.
    ///
    /// TODO: To implement stacking contexts correctly, we need to create a set of display lists,
    /// one per layer of the stacking context (CSS 2.1 § 9.9.1). Each box is passed the list set
    /// representing the box's stacking context. When asked to construct its constituent display
    /// items, each box puts its display items into the correct stack layer according to CSS 2.1
    /// Appendix E. Finally, the builder flattens the list.
    pub fn build_display_list<E:ExtraDisplayListData>(
                              &self,
                              builder: &DisplayListBuilder,
                              dirty: &Rect<Au>,
                              flow_origin: Point2D<Au>,
                              flow: &Flow,
                              index: uint,
                              lists: &RefCell<DisplayListCollection<E>>) {
        // Box position wrt to the owning flow.
        let box_bounds = self.border_box.get();
        let absolute_box_bounds = box_bounds.translate(&flow_origin);
        debug!("Box::build_display_list at rel={}, abs={}: {:s}",
               box_bounds, absolute_box_bounds, self.debug_str());
        debug!("Box::build_display_list: dirty={}, flow_origin={}", *dirty, flow_origin);

        if self.style().InheritedBox.get().visibility != visibility::visible {
            return;
        }

        if absolute_box_bounds.intersects(dirty) {
            debug!("Box::build_display_list: intersected. Adding display item...");
        } else {
            debug!("Box::build_display_list: Did not intersect...");
            return;
        }

        self.paint_inline_background_border_if_applicable(index, lists, &absolute_box_bounds, &flow_origin);
        // Add the background to the list, if applicable.
        self.paint_background_if_applicable(builder, index, lists, &absolute_box_bounds);

        // Add a border, if applicable.
        //
        // TODO: Outlines.
        self.paint_borders_if_applicable(index, lists, &absolute_box_bounds);

        match self.specific {
            UnscannedTextBox(_) => fail!("Shouldn't see unscanned boxes here."),
            TableColumnBox(_) => fail!("Shouldn't see table column boxes here."),
            ScannedTextBox(ref text_box) => {
                let text_color = self.style().Color.get().color.to_gfx_color();

                // Set the various text display item flags.
                let mut flow_flags = flow::base(flow).flags_info.clone();

                let inline_info = self.inline_info.borrow();
                match inline_info.get() {
                    &Some(ref info) => {
                        for data in info.parent_info.rev_iter() {
                            let parent_info = FlowFlagsInfo::new(data.style.get());
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
                lists.with_mut(|lists| {
                    let text_display_item = ~TextDisplayItem {
                        base: BaseDisplayItem {
                            bounds: bounds,
                            extra: ExtraDisplayListData::new(self),
                        },
                        text_run: text_box.run.clone(),
                        range: text_box.range,
                        text_color: text_color,
                        overline_color: flow_flags.overline_color(text_color),
                        underline_color: flow_flags.underline_color(text_color),
                        line_through_color: flow_flags.line_through_color(text_color),
                        flags: text_flags,
                    };

                    lists.lists[index].append_item(TextDisplayItemClass(text_display_item));
                });

                // Draw debug frames for text bounds.
                //
                // FIXME(pcwalton): This is a bit of an abuse of the logging infrastructure. We
                // should have a real `SERVO_DEBUG` system.
                debug!("{:?}", {
                    // Compute the text box bounds and draw a border surrounding them.
                    let debug_border = SideOffsets2D::new_all_same(Au::from_px(1));

                    lists.with_mut(|lists| {
                        let border_display_item = ~BorderDisplayItem {
                            base: BaseDisplayItem {
                                bounds: absolute_box_bounds,
                                extra: ExtraDisplayListData::new(self),
                            },
                            border: debug_border,
                            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
                            style: SideOffsets2D::new_all_same(border_style::solid)

                        };
                        lists.lists[index].append_item(BorderDisplayItemClass(border_display_item));
                    });

                    // Draw a rectangle representing the baselines.
                    let ascent = text_box.run.get().metrics_for_range(
                        &text_box.range).ascent;
                    let baseline = Rect(absolute_box_bounds.origin + Point2D(Au(0), ascent),
                                        Size2D(absolute_box_bounds.size.width, Au(0)));

                    lists.with_mut(|lists| {
                        let line_display_item = ~LineDisplayItem {
                            base: BaseDisplayItem {
                                bounds: baseline,
                                extra: ExtraDisplayListData::new(self),
                            },
                            color: rgb(0, 200, 0),
                            style: border_style::dashed

                        };
                        lists.lists[index].append_item(LineDisplayItemClass(line_display_item));
                    });
                });
            },
            GenericBox | IframeBox(..) | TableBox | TableCellBox | TableRowBox |
            TableWrapperBox => {
                lists.with_mut(|lists| {
                    let item = ~ClipDisplayItem {
                        base: BaseDisplayItem {
                            bounds: absolute_box_bounds,
                            extra: ExtraDisplayListData::new(self),
                        },
                        child_list: ~[],
                        need_clip: self.needs_clip()
                    };
                    lists.lists[index].append_item(ClipDisplayItemClass(item));
                });

                // FIXME(pcwalton): This is a bit of an abuse of the logging infrastructure. We
                // should have a real `SERVO_DEBUG` system.
                debug!("{:?}", {
                    // This prints a debug border around the border of this box.
                    let debug_border = SideOffsets2D::new_all_same(Au::from_px(1));

                    lists.with_mut(|lists| {
                        let border_display_item = ~BorderDisplayItem {
                            base: BaseDisplayItem {
                                bounds: absolute_box_bounds,
                                extra: ExtraDisplayListData::new(self),
                            },
                            border: debug_border,
                            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
                            style: SideOffsets2D::new_all_same(border_style::solid)

                        };
                        lists.lists[index].append_item(BorderDisplayItemClass(border_display_item));
                    });
                });
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

                match image_ref.get().get_image() {
                    Some(image) => {
                        debug!("(building display list) building image box");

                        // Place the image into the display list.
                        lists.with_mut(|lists| {
                            let image_display_item = ~ImageDisplayItem {
                                base: BaseDisplayItem {
                                    bounds: bounds,
                                    extra: ExtraDisplayListData::new(self),
                                },
                                image: image.clone(),
                            };
                            lists.lists[index].append_item(ImageDisplayItemClass(image_display_item));
                        });
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
                debug!("{:?}", {
                    let debug_border = SideOffsets2D::new_all_same(Au::from_px(1));

                    lists.with_mut(|lists| {
                        let border_display_item = ~BorderDisplayItem {
                            base: BaseDisplayItem {
                                bounds: absolute_box_bounds,
                                extra: ExtraDisplayListData::new(self),
                            },
                            border: debug_border,
                            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
                            style: SideOffsets2D::new_all_same(border_style::solid)

                        };
                        lists.lists[index].append_item(BorderDisplayItemClass(border_display_item))
                    });
                });

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

    /// Returns the *minimum width* and *preferred width* of this box as defined by CSS 2.1.
    pub fn minimum_and_preferred_widths(&self) -> (Au, Au) {
        let guessed_width = self.guess_width();
        let (additional_minimum, additional_preferred) = match self.specific {
            GenericBox | IframeBox(_) | TableBox | TableCellBox | TableColumnBox(_) |
            TableRowBox | TableWrapperBox => (Au(0), Au(0)),
            ImageBox(ref image_box_info) => {
                let image_width = image_box_info.image_width();
                (image_width, image_width)
            }
            ScannedTextBox(ref text_box_info) => {
                let range = &text_box_info.range;
                let min_line_width = text_box_info.run.get().min_width_for_range(range);

                let mut max_line_width = Au::new(0);
                for line_range in text_box_info.run.get().iter_natural_lines_for_range(range) {
                    let line_metrics = text_box_info.run.get().metrics_for_range(&line_range);
                    max_line_width = Au::max(max_line_width, line_metrics.advance_width);
                }

                (min_line_width, max_line_width)
            }
            UnscannedTextBox(..) => fail!("Unscanned text boxes should have been scanned by now!"),
        };
        (guessed_width + additional_minimum, guessed_width + additional_preferred)
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
                let text_bounds = run.get().metrics_for_range(range).bounding_box;
                text_bounds.size.width
            }
            TableColumnBox(_) => fail!("Table column boxes do not have width"),
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
        }
    }
    /// Returns, and computes, the height of this box.
    ///
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
                let text_bounds = run.get().metrics_for_range(range).bounding_box;
                let em_size = text_bounds.size.height;
                self.calculate_line_height(em_size)
            }
            TableColumnBox(_) => fail!("Table column boxes do not have height"),
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
        }
    }

    /// Return the size of the content box.
    pub fn content_box_size(&self) -> Size2D<Au> {
        let border_box_size = self.border_box.get().size;
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
                    let new_metrics = new_text_box_info.run.get().metrics_for_range(&left_range);
                    let mut new_box = self.transform(new_metrics.bounding_box.size, ScannedTextBox(new_text_box_info));
                    new_box.new_line_pos = ~[];
                    Some(new_box)
                };

                // Right box is for right text of first founded new-line character.
                let right_box = if right_range.length() > 0 {
                    let new_text_box_info = ScannedTextBoxInfo::new(text_box_info.run.clone(), right_range);
                    let new_metrics = new_text_box_info.run.get().metrics_for_range(&right_range);
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
                       text_box_info.run.get().text.get().len(),
                       text_box_info.range,
                       max_width);

                for (glyphs, offset, slice_range) in text_box_info.run.get().iter_slices_for_range(
                        &text_box_info.range) {
                    debug!("split_to_width: considering slice (offset={}, range={}, \
                                                               remain_width={})",
                           offset,
                           slice_range,
                           remaining_width);

                    let metrics = text_box_info.run.get().metrics_for_slice(glyphs, &slice_range);
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
                    let mut new_metrics = new_text_box_info.run.get().metrics_for_range(&left_range);
                    new_metrics.bounding_box.size.height = self.border_box.get().size.height;
                    Some(self.transform(new_metrics.bounding_box.size,
                                        ScannedTextBox(new_text_box_info)))
                } else {
                    None
                };

                let right_box = right_range.map_or(None, |range: Range| {
                    let new_text_box_info = ScannedTextBoxInfo::new(text_box_info.run.clone(), range);
                    let mut new_metrics = new_text_box_info.run.get().metrics_for_range(&range);
                    new_metrics.bounding_box.size.height = self.border_box.get().size.height;
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

    /// Assigns replaced width for this box only if it is replaced content.
    ///
    /// This assigns only the width, not margin or anything else.
    /// CSS 2.1 § 10.3.2.
    pub fn assign_replaced_width_if_necessary(&self,container_width: Au) {
        match self.specific {
            GenericBox | IframeBox(_) | TableBox | TableCellBox | TableRowBox |
            TableWrapperBox => {}
            ImageBox(ref image_box_info) => {
                // TODO(ksh8281): compute border,margin,padding
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
                position.get().size.width = width + self.noncontent_width() +
                    self.noncontent_inline_left() + self.noncontent_inline_right();
                image_box_info.computed_width.set(Some(width));
            }
            ScannedTextBox(_) => {
                // Scanned text boxes will have already had their
                // content_widths assigned by this point.
                let mut position = self.border_box.borrow_mut();
                position.get().size.width = position.get().size.width + self.noncontent_width() +
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
                image_box_info.computed_height.set(Some(height));
                position.get().size.height = height + self.noncontent_height()
            }
            ScannedTextBox(_) => {
                // Scanned text boxes will have already had their widths assigned by this point
                let mut position = self.border_box.borrow_mut();
                // Scanned text boxes' content heights are calculated by the
                // text run scanner during Flow construction.
                position.get().size.height
                    = position.get().size.height + self.noncontent_height()
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
            ScannedTextBox(ref text_box_info) => text_box_info.run.get().teardown(),
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
                self.side_offsets_debug_string("b", self.border.get()),
                self.side_offsets_debug_string("p", self.padding.get()),
                self.side_offsets_debug_string("m", self.margin.get()))
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
        let left = offset.x + self.margin.get().left + self.border.get().left +
            self.padding.get().left;
        let top = offset.y + self.margin.get().top + self.border.get().top +
            self.padding.get().top;
        let width = self.border_box.get().size.width - self.noncontent_width();
        let height = self.border_box.get().size.height - self.noncontent_height();
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
