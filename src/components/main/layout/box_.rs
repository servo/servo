/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Box` type, which represents the leaves of the layout tree.

use extra::url::Url;
use extra::arc::{MutexArc, Arc};
use geom::{Point2D, Rect, Size2D, SideOffsets2D};
use gfx::color::rgb;
use gfx::display_list::{BaseDisplayItem, BorderDisplayItem, BorderDisplayItemClass};
use gfx::display_list::{DisplayList, ImageDisplayItem, ImageDisplayItemClass};
use gfx::display_list::{SolidColorDisplayItem, SolidColorDisplayItemClass, TextDisplayItem};
use gfx::display_list::{TextDisplayItemClass, TextDisplayItemFlags, ClipDisplayItem};
use gfx::display_list::{ClipDisplayItemClass};
use gfx::font::FontStyle;

use gfx::text::text_run::TextRun;
use servo_msg::constellation_msg::{FrameRectMsg, PipelineId, SubpageId};
use servo_net::image::holder::ImageHolder;
use servo_net::local_image_cache::LocalImageCache;
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::range::*;
use servo_util::namespace;
use std::cast;
use std::cell::RefCell;
use std::cmp::ApproxEq;
use std::num::Zero;
use style::{ComputedValues, TElement, TNode, cascade};
use style::computed_values::{LengthOrPercentage, LengthOrPercentageOrAuto, overflow, LPA_Auto};
use style::computed_values::{border_style, clear, font_family, line_height};
use style::computed_values::{text_align, text_decoration, vertical_align, visibility, white_space};

use css::node_style::StyledNode;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData, ToGfxColor};
use layout::float_context::{ClearType, ClearLeft, ClearRight, ClearBoth};
use layout::flow::{Flow, FlowFlagsInfo};
use layout::flow;
use layout::model::{MaybeAuto, specified, Auto, Specified};
use layout::util::OpaqueNode;
use layout::wrapper::LayoutNode;

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
    position: RefCell<Rect<Au>>,

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
    pub fn new(node: &LayoutNode, image_url: Url, local_image_cache: MutexArc<LocalImageCache>)
               -> ImageBoxInfo {

        fn convert_length(node: &LayoutNode, name: &str) -> Option<Au> {
            node.with_element(|element| {
                element.get_attr(&namespace::Null, name).and_then(|string| {
                    let n: Option<int> = FromStr::from_str(string);
                    n
                }).and_then(|pixels| Some(Au::from_px(pixels)))
            })
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
    pub fn new(node: &LayoutNode) -> IframeBoxInfo {
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
    pub fn new(node: &LayoutNode) -> UnscannedTextBoxInfo {
        // FIXME(pcwalton): Don't copy text; atomically reference count it instead.
        UnscannedTextBoxInfo {
            text: node.text(),
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


impl Box {
    /// Constructs a new `Box` instance.
    pub fn new(node: LayoutNode, specific: SpecificBoxInfo) -> Box {
        // Find the nearest ancestor element and take its style. (It should be either that node or
        // its immediate parent.)
        // CSS 2.1 § 9.2.1.1,9.2.2.1 This is for non-inherited properties on anonymous boxes
        // example:
        //
        //     <div style="border: solid">
        //         <p>Foo</p>
        //         Bar
        //         <p>Baz</p>
        //     </div>
        //
        // An anonymous block box is generated around `Bar`, but it shouldn't inherit the border.

        let node_style = if node.is_element() {
            node.style().clone()
        } else {
            let mut nearest_ancestor_element = node;
            while !nearest_ancestor_element.is_element() {
                nearest_ancestor_element =
                    nearest_ancestor_element.parent_node().expect("no nearest element?!");
            }

            // Anonymous box: inheriting from the ancestor with no specified declarations.
            Arc::new(cascade(&[Arc::new(~[])],
                             Some(nearest_ancestor_element.style().get())))
        };

        Box {
            node: OpaqueNode::from_layout_node(&node),
            style: node_style,
            position: RefCell::new(Au::zero_rect()),
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
                    match MaybeAuto::from_style(self.style().Box.height, Au(0)) {
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
                    match MaybeAuto::from_style(self.style().Box.width, Au(0)) {
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
            position: RefCell::new(Rect(self.position.get().origin, size)),
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
        match self.specific {
            GenericBox | IframeBox(_) | ImageBox(_) => {}
            ScannedTextBox(_) | UnscannedTextBox(_) => return Au(0),
        }

        let style = self.style();
        let width = MaybeAuto::from_style(style.Box.width, Au::new(0)).specified_or_zero();
        let margin_left = MaybeAuto::from_style(style.Margin.margin_left,
                                                Au::new(0)).specified_or_zero();
        let margin_right = MaybeAuto::from_style(style.Margin.margin_right,
                                                 Au::new(0)).specified_or_zero();

        let padding_left = self.compute_padding_length(style.Padding.padding_left, Au::new(0));
        let padding_right = self.compute_padding_length(style.Padding.padding_right, Au::new(0));

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
        #[inline]
        fn width(width: Au, style: border_style::T) -> Au {
            if style == border_style::none {
                Au(0)
            } else {
                width
            }
        }

        self.border.set(SideOffsets2D::new(width(style.Border.border_top_width,
                                                 style.Border.border_top_style),
                                           width(style.Border.border_right_width,
                                                 style.Border.border_right_style),
                                           width(style.Border.border_bottom_width,
                                                 style.Border.border_bottom_style),
                                           width(style.Border.border_left_width,
                                                 style.Border.border_left_style)))
    }

    pub fn compute_positioned_offsets(&self, style: &ComputedValues, containing_width: Au, containing_height: Au) {
        self.position_offsets.set(SideOffsets2D::new(
                MaybeAuto::from_style(style.PositionOffsets.top, containing_height)
                .specified_or_zero(),
                MaybeAuto::from_style(style.PositionOffsets.right, containing_width)
                .specified_or_zero(),
                MaybeAuto::from_style(style.PositionOffsets.bottom, containing_height)
                .specified_or_zero(),
                MaybeAuto::from_style(style.PositionOffsets.left, containing_width)
                .specified_or_zero()));
    }

    /// Populates the box model padding parameters from the given computed style.
    pub fn compute_padding(&self, style: &ComputedValues, containing_block_width: Au) {
        let padding = SideOffsets2D::new(self.compute_padding_length(style.Padding.padding_top,
                                                                     containing_block_width),
                                         self.compute_padding_length(style.Padding.padding_right,
                                                                     containing_block_width),
                                         self.compute_padding_length(style.Padding.padding_bottom,
                                                                     containing_block_width),
                                         self.compute_padding_length(style.Padding.padding_left,
                                                                     containing_block_width));
        self.padding.set(padding)
    }

    fn compute_padding_length(&self, padding: LengthOrPercentage, content_box_width: Au) -> Au {
        specified(padding, content_box_width)
    }

    pub fn noncontent_width(&self) -> Au {
        self.noncontent_left() + self.noncontent_right()
    }

    pub fn noncontent_height(&self) -> Au {
        self.noncontent_top() + self.noncontent_bottom()
    }

    pub fn noncontent_left(&self) -> Au {
        self.margin.get().left + self.border.get().left + self.padding.get().left
    }

    pub fn noncontent_right(&self) -> Au {
        self.margin.get().right + self.border.get().right + self.padding.get().right
    }

    pub fn noncontent_top(&self) -> Au {
        self.margin.get().top + self.border.get().top + self.padding.get().top
    }

    pub fn noncontent_bottom(&self) -> Au {
        self.margin.get().bottom + self.border.get().bottom + self.padding.get().bottom
    }

    pub fn noncontent_inline_left(&self) -> Au {
        let mut left = Au::new(0);
        let info = self.inline_info.borrow();
        match info.get() {
            &Some(ref info) => {
                for info in info.parent_info.iter() {
                    left = left + info.margin.left + info.border.left + info.padding.left;
                }
            },
            &None => {}
        }
        left
    }

    pub fn noncontent_inline_right(&self) -> Au {
        let mut right = Au::new(0);
        let info = self.inline_info.borrow();
        match info.get() {
            &Some(ref info) => {
                for info in info.parent_info.iter() {
                    right = right + info.margin.right + info.border.right + info.padding.right;
                }
            },
            &None => {}
        }
        right
    }

    pub fn noncontent_inline_top(&self) -> Au {
        let mut top = Au::new(0);
        let info = self.inline_info.borrow();
        match info.get() {
            &Some(ref info) => {
                for info in info.parent_info.iter() {
                    top = top + info.margin.top + info.border.top + info.padding.top;
                }
            },
            &None => {}
        }
        top
    }

    pub fn noncontent_inline_bottom(&self) -> Au {
        let mut bottom = Au::new(0);
        let info = self.inline_info.borrow();
        match info.get() {
            &Some(ref info) => {
                for info in info.parent_info.iter() {
                    bottom = bottom + info.margin.bottom + info.border.bottom + info.padding.bottom;
                }
            },
            &None => {}
        }
        bottom
    }

    pub fn merge_noncontent_inline_right(&self, other_box: &Box) {
        let mut info = self.inline_info.borrow_mut();
        let other_info = other_box.inline_info.borrow();

        match other_info.get() {
            &Some(ref other_info) => {
                match info.get() {
                    &Some(ref mut info) => {
                        for other_item in other_info.parent_info.iter() {
                            for item in info.parent_info.mut_iter() {
                                if item.node == other_item.node {
                                    item.border.right = other_item.border.right;
                                    item.padding.right = other_item.padding.right;
                                    item.margin.right = other_item.margin.right;
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

    pub fn merge_noncontent_inline_left(&self, other_box: &Box) {
        let mut info = self.inline_info.borrow_mut();
        let other_info = other_box.inline_info.borrow();

        match other_info.get() {
            &Some(ref other_info) => {
                match info.get() {
                    &Some(ref mut info) => {
                        for other_item in other_info.parent_info.iter() {
                            for item in info.parent_info.mut_iter() {
                                if item.node == other_item.node {
                                    item.border.left = other_item.border.left;
                                    item.padding.left = other_item.padding.left;
                                    item.margin.left = other_item.margin.left;
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

    pub fn clear_noncontent_inline_right(&self) {
        let mut info = self.inline_info.borrow_mut();
        match info.get() {
            &Some(ref mut info) => {
                for item in info.parent_info.mut_iter() {
                    item.border.right = Au::new(0);
                    item.padding.right = Au::new(0);
                    item.margin.right = Au::new(0);
                }
            },
            &None => {}
        }
    }

    pub fn clear_noncontent_inline_left(&self) {
        let mut info = self.inline_info.borrow_mut();
        match info.get() {
            &Some(ref mut info) => {
                for item in info.parent_info.mut_iter() {
                    item.border.left = Au::new(0);
                    item.padding.left = Au::new(0);
                    item.margin.left = Au::new(0);
                }
            },
            &None => {}
        }
    }
    /// Always inline for SCCP.
    ///
    /// FIXME(pcwalton): Just replace with the clear type from the style module for speed?
    #[inline(always)]
    pub fn clear(&self) -> Option<ClearType> {
        let style = self.style();
        match style.Box.clear {
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
        let font_families = my_style.Font.font_family.map(|family| {
            match *family {
                font_family::FamilyName(ref name) => (*name).clone(),
            }
        });
        debug!("(font style) font families: `{:?}`", font_families);

        let font_size = my_style.Font.font_size.to_f64().unwrap() / 60.0;
        debug!("(font style) font size: `{:f}px`", font_size);

        FontStyle {
            pt_size: font_size,
            weight: my_style.Font.font_weight,
            style: my_style.Font.font_style,
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
        self.style().Text.text_align
    }

    pub fn line_height(&self) -> line_height::T {
        self.style().Box.line_height
    }

    pub fn vertical_align(&self) -> vertical_align::T {
        self.style().Box.vertical_align
    }

    pub fn white_space(&self) -> white_space::T {
        self.style().Text.white_space
    }

    /// Returns the text decoration of this box, according to the style of the nearest ancestor
    /// element.
    ///
    /// NB: This may not be the actual text decoration, because of the override rules specified in
    /// CSS 2.1 § 16.3.1. Unfortunately, computing this properly doesn't really fit into Servo's
    /// model. Therefore, this is a best lower bound approximation, but the end result may actually
    /// have the various decoration flags turned on afterward.
    pub fn text_decoration(&self) -> text_decoration::T {
        self.style().Text.text_decoration
    }

    /// Returns the sum of margin, border, and padding on the left.
    pub fn offset(&self) -> Au {
        self.margin.get().left + self.border.get().left + self.padding.get().left
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
                                          list: &RefCell<DisplayList<E>>,
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
                        info.style.get().Background.background_color);

                    if !background_color.alpha.approx_eq(&0.0) {
                        list.with_mut(|list| {
                            let solid_color_display_item = ~SolidColorDisplayItem {
                                base: BaseDisplayItem {
                                          bounds: bg_rect.clone(),
                                          extra: ExtraDisplayListData::new(self),
                                      },
                                      color: background_color.to_gfx_color(),
                            };

                            list.append_item(SolidColorDisplayItemClass(solid_color_display_item))
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
                    let top_color = style.resolve_color(style.Border.border_top_color);
                    let right_color = style.resolve_color(style.Border.border_right_color);
                    let bottom_color = style.resolve_color(style.Border.border_bottom_color);
                    let left_color = style.resolve_color(style.Border.border_left_color);
                    let top_style = style.Border.border_top_style;
                    let right_style = style.Border.border_right_style;
                    let bottom_style = style.Border.border_bottom_style;
                    let left_style = style.Border.border_left_style;


                    list.with_mut(|list| {
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

                        list.append_item(BorderDisplayItemClass(border_display_item))
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
                                          list: &RefCell<DisplayList<E>>,
                                          absolute_bounds: &Rect<Au>) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a box".
        let style = self.style();
        let background_color = style.resolve_color(style.Background.background_color);
        if !background_color.alpha.approx_eq(&0.0) {
            list.with_mut(|list| {
                let solid_color_display_item = ~SolidColorDisplayItem {
                    base: BaseDisplayItem {
                        bounds: *absolute_bounds,
                        extra: ExtraDisplayListData::new(self),
                    },
                    color: background_color.to_gfx_color(),
                };

                list.append_item(SolidColorDisplayItemClass(solid_color_display_item))
            })
        }
    }

    /// Adds the display items necessary to paint the borders of this box to a display list if
    /// necessary.
    pub fn paint_borders_if_applicable<E:ExtraDisplayListData>(
                                       &self,
                                       list: &RefCell<DisplayList<E>>,
                                       abs_bounds: &Rect<Au>) {
        // Fast path.
        let border = self.border.get();
        if border.is_zero() {
            return
        }

        let style = self.style();
        let top_color = style.resolve_color(style.Border.border_top_color);
        let right_color = style.resolve_color(style.Border.border_right_color);
        let bottom_color = style.resolve_color(style.Border.border_bottom_color);
        let left_color = style.resolve_color(style.Border.border_left_color);
        let top_style = style.Border.border_top_style;
        let right_style = style.Border.border_right_style;
        let bottom_style = style.Border.border_bottom_style;
        let left_style = style.Border.border_left_style;

        let mut abs_bounds = abs_bounds.clone();
        abs_bounds.origin.x = abs_bounds.origin.x + self.noncontent_inline_left();
        abs_bounds.size.width = abs_bounds.size.width - self.noncontent_inline_left()
            - self.noncontent_inline_right();

        // Append the border to the display list.
        list.with_mut(|list| {
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

            list.append_item(BorderDisplayItemClass(border_display_item))
        });
    }

    /// Adds the display items for this box to the given display list.
    ///
    /// Arguments:
    /// * `builder`: The display list builder, which manages the coordinate system and options.
    /// * `dirty`: The dirty rectangle in the coordinate system of the owning flow.
    /// * `origin`: The total offset from the display list root flow to the owning flow of this
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
                              offset: Point2D<Au>,
                              flow: &Flow,
                              list: &RefCell<DisplayList<E>>) {
        let box_bounds = self.position.get();
        let absolute_box_bounds = box_bounds.translate(&offset);
        debug!("Box::build_display_list at rel={}, abs={}: {:s}",
               box_bounds, absolute_box_bounds, self.debug_str());
        debug!("Box::build_display_list: dirty={}, offset={}", *dirty, offset);

        if self.style().Box.visibility != visibility::visible {
            return;
        }

        if absolute_box_bounds.intersects(dirty) {
            debug!("Box::build_display_list: intersected. Adding display item...");
        } else {
            debug!("Box::build_display_list: Did not intersect...");
            return;
        }

        self.paint_inline_background_border_if_applicable(list, &absolute_box_bounds, &offset);
        // Add the background to the list, if applicable.
        self.paint_background_if_applicable(list, &absolute_box_bounds);

        match self.specific {
            UnscannedTextBox(_) => fail!("Shouldn't see unscanned boxes here."),
            ScannedTextBox(ref text_box) => {
                list.with_mut(|list| {
                    let item = ~ClipDisplayItem {
                        base: BaseDisplayItem {
                            bounds: absolute_box_bounds,
                            extra: ExtraDisplayListData::new(self),
                        },
                        child_list: ~[],
                        need_clip: false
                    };
                    list.append_item(ClipDisplayItemClass(item));
                });

                let text_color = self.style().Color.color.to_gfx_color();

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
                list.with_mut(|list| {
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

                    list.append_item(TextDisplayItemClass(text_display_item))
                });

                // Draw debug frames for text bounds.
                //
                // FIXME(pcwalton): This is a bit of an abuse of the logging infrastructure. We
                // should have a real `SERVO_DEBUG` system.
                debug!("{:?}", {
                    // Compute the text box bounds and draw a border surrounding them.
                    let debug_border = SideOffsets2D::new_all_same(Au::from_px(1));

                    list.with_mut(|list| {
                        let border_display_item = ~BorderDisplayItem {
                            base: BaseDisplayItem {
                                bounds: absolute_box_bounds,
                                extra: ExtraDisplayListData::new(self),
                            },
                            border: debug_border,
                            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
                            style: SideOffsets2D::new_all_same(border_style::solid)

                        };
                        list.append_item(BorderDisplayItemClass(border_display_item))
                    });

                    // Draw a rectangle representing the baselines.
                    //
                    // TODO(Issue #221): Create and use a Line display item for the baseline.
                    let ascent = text_box.run.get().metrics_for_range(
                        &text_box.range).ascent;
                    let baseline = Rect(absolute_box_bounds.origin + Point2D(Au(0), ascent),
                                        Size2D(absolute_box_bounds.size.width, Au(0)));

                    list.with_mut(|list| {
                        let border_display_item = ~BorderDisplayItem {
                            base: BaseDisplayItem {
                                bounds: baseline,
                                extra: ExtraDisplayListData::new(self),
                            },
                            border: debug_border,
                            color: SideOffsets2D::new_all_same(rgb(0, 200, 0)),
                            style: SideOffsets2D::new_all_same(border_style::dashed)

                        };
                        list.append_item(BorderDisplayItemClass(border_display_item))
                    });
                });
            },
            GenericBox | IframeBox(..) => {
                list.with_mut(|list| {
                    let item = ~ClipDisplayItem {
                        base: BaseDisplayItem {
                            bounds: absolute_box_bounds,
                            extra: ExtraDisplayListData::new(self),
                        },
                        child_list: ~[],
                        need_clip: self.needs_clip()
                    };
                    list.append_item(ClipDisplayItemClass(item));
                });

                // FIXME(pcwalton): This is a bit of an abuse of the logging infrastructure. We
                // should have a real `SERVO_DEBUG` system.
                debug!("{:?}", {
                    let debug_border = SideOffsets2D::new_all_same(Au::from_px(1));

                    list.with_mut(|list| {
                        let border_display_item = ~BorderDisplayItem {
                            base: BaseDisplayItem {
                                bounds: absolute_box_bounds,
                                extra: ExtraDisplayListData::new(self),
                            },
                            border: debug_border,
                            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
                            style: SideOffsets2D::new_all_same(border_style::solid)

                        };
                        list.append_item(BorderDisplayItemClass(border_display_item))
                    });
                });
            },
            ImageBox(ref image_box) => {
                list.with_mut(|list| {
                    let item = ~ClipDisplayItem {
                        base: BaseDisplayItem {
                            bounds: absolute_box_bounds,
                            extra: ExtraDisplayListData::new(self),
                        },
                        child_list: ~[],
                        need_clip: false
                    };
                    list.append_item(ClipDisplayItemClass(item));
                });

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
                        list.with_mut(|list| {
                            let image_display_item = ~ImageDisplayItem {
                                base: BaseDisplayItem {
                                    bounds: bounds,
                                    extra: ExtraDisplayListData::new(self),
                                },
                                image: image.clone(),
                            };
                            list.append_item(ImageDisplayItemClass(image_display_item));
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

                    list.with_mut(|list| {
                        let border_display_item = ~BorderDisplayItem {
                            base: BaseDisplayItem {
                                bounds: absolute_box_bounds,
                                extra: ExtraDisplayListData::new(self),
                            },
                            border: debug_border,
                            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
                            style: SideOffsets2D::new_all_same(border_style::solid)

                        };
                        list.append_item(BorderDisplayItemClass(border_display_item))
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
                self.finalize_position_and_size_of_iframe(iframe_box, offset, builder.ctx)
            }
            GenericBox | ImageBox(_) | ScannedTextBox(_) | UnscannedTextBox(_) => {}
        }

        // Add a border, if applicable.
        //
        // TODO: Outlines.
        self.paint_borders_if_applicable(list, &absolute_box_bounds);
    }

    /// Returns the *minimum width* and *preferred width* of this box as defined by CSS 2.1.
    pub fn minimum_and_preferred_widths(&self) -> (Au, Au) {
        let guessed_width = self.guess_width();
        let (additional_minimum, additional_preferred) = match self.specific {
            GenericBox | IframeBox(_) => (Au(0), Au(0)),
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


    pub fn content_width(&self) -> Au {
        match self.specific {
            GenericBox | IframeBox(_) => Au(0),
            ImageBox(ref image_box_info) => {
                image_box_info.computed_width()
            }
            ScannedTextBox(ref text_box_info) => {
                let (range, run) = (&text_box_info.range, &text_box_info.run);
                let text_bounds = run.get().metrics_for_range(range).bounding_box;
                text_bounds.size.width
            }
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
        }
    }
    /// Returns, and computes, the height of this box.
    ///
    pub fn content_height(&self) -> Au {
        match self.specific {
            GenericBox | IframeBox(_) => Au(0),
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
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
        }
    }

    /// Split box which includes new-line character
    pub fn split_by_new_line(&self) -> SplitBoxResult {
        match self.specific {
            GenericBox | IframeBox(_) | ImageBox(_) => CannotSplit,
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
            ScannedTextBox(ref text_box_info) => {
                let mut new_line_pos = self.new_line_pos.clone();
                let cur_new_line_pos = new_line_pos.shift();

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
            GenericBox | IframeBox(_) | ImageBox(_) => CannotSplit,
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
                    new_metrics.bounding_box.size.height = self.position.get().size.height;
                    Some(self.transform(new_metrics.bounding_box.size,
                                        ScannedTextBox(new_text_box_info)))
                } else {
                    None
                };

                let right_box = right_range.map_default(None, |range: Range| {
                    let new_text_box_info = ScannedTextBoxInfo::new(text_box_info.run.clone(), range);
                    let mut new_metrics = new_text_box_info.run.get().metrics_for_range(&range);
                    new_metrics.bounding_box.size.height = self.position.get().size.height;
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
            UnscannedTextBox(ref text_box_info) => text_box_info.text.is_whitespace(),
            _ => false,
        }
    }

    /// Assigns the appropriate width to this box.
    pub fn assign_width(&self,container_width: Au) {
        match self.specific {
            GenericBox | IframeBox(_) => {
            }
            ImageBox(ref image_box_info) => {
                // TODO(ksh8281): compute border,margin,padding
                let width = ImageBoxInfo::style_length(self.style().Box.width,
                                                       image_box_info.dom_width,
                                                       container_width);

                // FIXME(ksh8281): we shouldn't figure height this way
                // now, we don't know about size of parent's height
                let height = ImageBoxInfo::style_length(self.style().Box.height,
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

                let mut position = self.position.borrow_mut();
                position.get().size.width = width + self.noncontent_width() +
                    self.noncontent_inline_left() + self.noncontent_inline_right();
                image_box_info.computed_width.set(Some(width));
            }
            ScannedTextBox(_) => {
                // Scanned text boxes will have already had their content_widths assigned by this point.
                let mut position = self.position.borrow_mut();
                position.get().size.width = position.get().size.width + self.noncontent_width() +
                    self.noncontent_inline_left() + self.noncontent_inline_right();
            }
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
        }
    }

    pub fn assign_height(&self) {
        match self.specific {
            GenericBox | IframeBox(_) => {
            }
            ImageBox(ref image_box_info) => {
                // TODO(ksh8281): compute border,margin,padding
                let width = image_box_info.computed_width();
                // FIXME(ksh8281): we shouldn't assign height this way
                // we don't know about size of parent's height
                let height = ImageBoxInfo::style_length(self.style().Box.height,
                                                        image_box_info.dom_height,
                                                        Au::new(0));

                let height = match (self.style().Box.width, 
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

                let mut position = self.position.borrow_mut();
                image_box_info.computed_height.set(Some(height));
                position.get().size.height = height + self.noncontent_height()
            }
            ScannedTextBox(_) => {
                // Scanned text boxes will have already had their widths assigned by this point
                let mut position = self.position.borrow_mut();
                position.get().size.height
                    = position.get().size.height + self.noncontent_height()
            }
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
        self.style().Box.overflow == overflow::hidden
    }

    /// Returns a debugging string describing this box.
    pub fn debug_str(&self) -> ~str {
        let class_name = match self.specific {
            GenericBox => "GenericBox",
            IframeBox(_) => "IframeBox",
            ImageBox(_) => "ImageBox",
            ScannedTextBox(_) => "ScannedTextBox",
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
                *value.top,
                *value.right,
                *value.bottom,
                *value.left)
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
        let width = self.position.get().size.width - self.noncontent_width();
        let height = self.position.get().size.height - self.noncontent_height();
        let origin = Point2D(geometry::to_frac_px(left) as f32, geometry::to_frac_px(top) as f32);
        let size = Size2D(geometry::to_frac_px(width) as f32, geometry::to_frac_px(height) as f32);
        let rect = Rect(origin, size);

        debug!("finalizing position and size of iframe for {:?},{:?}",
               iframe_box.pipeline_id,
               iframe_box.subpage_id);
        let msg = FrameRectMsg(iframe_box.pipeline_id, iframe_box.subpage_id, rect);
        layout_context.constellation_chan.send(msg)
    }
}

