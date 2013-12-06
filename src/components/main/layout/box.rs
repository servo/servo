/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Box` type, which represents the leaves of the layout tree.

use extra::url::Url;
use extra::arc::MutexArc;
use geom::{Point2D, Rect, Size2D, SideOffsets2D};
use gfx::color::rgb;
use gfx::display_list::{BaseDisplayItem, BorderDisplayItem, BorderDisplayItemClass};
use gfx::display_list::{DisplayList, ImageDisplayItem, ImageDisplayItemClass};
use gfx::display_list::{SolidColorDisplayItem, SolidColorDisplayItemClass, TextDisplayItem};
use gfx::display_list::{TextDisplayItemClass, ClipDisplayItem, ClipDisplayItemClass};
use gfx::font::{FontStyle, FontWeight300};
use gfx::text::text_run::TextRun;
use script::dom::node::{AbstractNode, LayoutView};
use servo_net::image::holder::ImageHolder;
use servo_net::local_image_cache::LocalImageCache;
use servo_util::geometry::Au;
use servo_util::range::*;
use servo_util::slot::Slot;
use servo_util::tree::{TreeNodeRef, ElementLike};
use std::cast;
use std::cell::Cell;
use std::cmp::ApproxEq;
use std::num::Zero;
use style::ComputedValues;
use style::computed_values::{LengthOrPercentage, overflow};
use style::computed_values::{border_style, clear, float, font_family, font_style, line_height};
use style::computed_values::{position, text_align, text_decoration, vertical_align, visibility};

use css::node_style::StyledNode;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData, ToGfxColor};
use layout::float_context::{ClearType, ClearLeft, ClearRight, ClearBoth};
use layout::model::{MaybeAuto, specified};

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
pub struct Box {
    /// The DOM node that this `Box` originates from.
    node: AbstractNode<LayoutView>,

    /// The position of this box relative to its owning flow.
    position: Slot<Rect<Au>>,

    /// The border of the content box.
    ///
    /// FIXME(pcwalton): This need not be stored in the box.
    border: Slot<SideOffsets2D<Au>>,

    /// The padding of the content box.
    padding: Slot<SideOffsets2D<Au>>,

    /// The margin of the content box.
    margin: Slot<SideOffsets2D<Au>>,

    /// The width of the content box.
    content_box_width: Au,

    /// Info specific to the kind of box. Keep this enum small.
    specific: SpecificBoxInfo,
}

/// Info specific to the kind of box. Keep this enum small.
pub enum SpecificBoxInfo {
    GenericBox,
    ImageBox(ImageBoxInfo),
    ScannedTextBox(ScannedTextBoxInfo),
    UnscannedTextBox(UnscannedTextBoxInfo),
}

/// A box that represents a replaced content image and its accompanying borders, shadows, etc.
pub struct ImageBoxInfo {
    /// The image held within this box.
    image: Slot<ImageHolder>,
}

impl ImageBoxInfo {
    /// Creates a new image box from the given URL and local image cache.
    ///
    /// FIXME(pcwalton): The fact that image boxes store the cache in the box makes little sense to
    /// me.
    pub fn new(image_url: Url, local_image_cache: MutexArc<LocalImageCache>) -> ImageBoxInfo {
        ImageBoxInfo {
            image: Slot::init(ImageHolder::new(image_url, local_image_cache)),
        }
    }

    // Calculate the width of an image, accounting for the width attribute
    // TODO: This could probably go somewhere else
    pub fn image_width(&self, base: &Box) -> Au {
        let attr_width: Option<int> = do base.node.with_imm_element |elt| {
            match elt.get_attr("width") {
                Some(width) => {
                    FromStr::from_str(width)
                }
                None => {
                    None
                }
            }
        };

        // TODO: Consult margins and borders?
        let px_width = if attr_width.is_some() {
            attr_width.unwrap()
        } else {
            self.image.mutate().ptr.get_size().unwrap_or(Size2D(0, 0)).width
        };

        Au::from_px(px_width)
    }

    // Calculate the height of an image, accounting for the height attribute
    // TODO: This could probably go somewhere else
    pub fn image_height(&self, base: &Box) -> Au {
        let attr_height: Option<int> = do base.node.with_imm_element |elt| {
            match elt.get_attr("height") {
                Some(height) => {
                    FromStr::from_str(height)
                }
                None => {
                    None
                }
            }
        };

        // TODO: Consult margins and borders?
        let px_height = if attr_height.is_some() {
            attr_height.unwrap()
        } else {
            self.image.mutate().ptr.get_size().unwrap_or(Size2D(0, 0)).height
        };

        Au::from_px(px_height)
    }
}

/// A scanned text box represents a single run of text with a distinct style. A `TextBox` may be
/// split into two or more boxes across line breaks. Several `TextBox`es may correspond to a single
/// DOM text node. Split text boxes are implemented by referring to subsets of a single `TextRun`
/// object.
pub struct ScannedTextBoxInfo {
    /// The text run that this represents.
    run: @TextRun,

    /// The range within the above text run that this represents.
    range: Range,
}

impl ScannedTextBoxInfo {
    /// Creates the information specific to a scanned text box from a range and a text run.
    pub fn new(run: @TextRun, range: Range) -> ScannedTextBoxInfo {
        ScannedTextBoxInfo {
            run: run,
            range: range,
        }
    }
}

/// Data for an unscanned text box. Unscanned text boxes are the results of flow construction that
/// have not yet had their width determined.
pub struct UnscannedTextBoxInfo {
    /// The text inside the box.
    text: ~str,
}

impl UnscannedTextBoxInfo {
    /// Creates a new instance of `UnscannedTextBoxInfo` from the given DOM node.
    pub fn new(node: &AbstractNode<LayoutView>) -> UnscannedTextBoxInfo {
        node.with_imm_text(|text_node| {
            // FIXME(pcwalton): Don't copy text; atomically reference count it instead.
            UnscannedTextBoxInfo {
                text: text_node.element.data.to_str(),
            }
        })
    }
}

/// Represents the outcome of attempting to split a box.
pub enum SplitBoxResult {
    CannotSplit(@Box),
    // in general, when splitting the left or right side can
    // be zero length, due to leading/trailing trimmable whitespace
    SplitDidFit(Option<@Box>, Option<@Box>),
    SplitDidNotFit(Option<@Box>, Option<@Box>)
}

impl Box {
    /// Constructs a new `Box` instance.
    pub fn new(node: AbstractNode<LayoutView>, specific: SpecificBoxInfo) -> Box {
        Box {
            node: node,
            position: Slot::init(Au::zero_rect()),
            border: Slot::init(Zero::zero()),
            padding: Slot::init(Zero::zero()),
            margin: Slot::init(Zero::zero()),
            content_box_width: Zero::zero(),
            specific: specific,
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
            position: Slot::init(Rect(self.position.get().origin, size)),
            border: Slot::init(self.border.get()),
            padding: Slot::init(self.padding.get()),
            margin: Slot::init(self.margin.get()),
            content_box_width: self.content_box_width,
            specific: specific,
        }
    }

    fn guess_width(&self) -> Au {
        if !self.node.is_element() {
            return Au(0)
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

    /// Sets the size of this box.
    fn set_size(&self, new_size: Size2D<Au>) {
        self.position.set(Rect(self.position.get().origin, new_size))
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
        self.border.set(SideOffsets2D::new(style.Border.border_top_width,
                                           style.Border.border_right_width,
                                           style.Border.border_bottom_width,
                                           style.Border.border_left_width))
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

    pub fn compute_padding_length(&self, padding: LengthOrPercentage, content_box_width: Au)
                                  -> Au {
        specified(padding, content_box_width)
    }

    pub fn noncontent_width(&self) -> Au {
        let left = self.margin.get().left + self.border.get().left + self.padding.get().left;
        let right = self.margin.get().right + self.border.get().right + self.padding.get().right;
        left + right
    }

    pub fn noncontent_height(&self) -> Au {
        let top = self.margin.get().top + self.border.get().top + self.padding.get().top;
        let bottom = self.margin.get().bottom + self.border.get().bottom +
            self.padding.get().bottom;
        top + bottom
    }

    /// The box formed by the content edge as defined in CSS 2.1 § 8.1. Coordinates are relative to
    /// the owning flow.
    pub fn content_box(&self) -> Rect<Au> {
        let position = self.position.get();
        let origin = Point2D(position.origin.x + self.border.get().left + self.padding.get().left,
                             position.origin.y);
        let noncontent_width = self.border.get().left + self.padding.get().left +
            self.border.get().right + self.padding.get().right;
        let size = Size2D(position.size.width - noncontent_width, position.size.height);
        Rect(origin, size)
    }

    /// The box formed by the border edge as defined in CSS 2.1 § 8.1. Coordinates are relative to
    /// the owning flow.
    pub fn border_box(&self) -> Rect<Au> {
        // TODO: Actually compute the content box, padding, and border.
        self.content_box()
    }

    /// The box formed by the margin edge as defined in CSS 2.1 § 8.1. Coordinates are relative to
    /// the owning flow.
    pub fn margin_box(&self) -> Rect<Au> {
        // TODO: Actually compute the content_box, padding, border, and margin.
        self.content_box()
    }

    /// Returns the nearest ancestor-or-self `Element` to the DOM node that this box represents.
    ///
    /// If there is no ancestor-or-self `Element` node, fails.
    pub fn nearest_ancestor_element(&self) -> AbstractNode<LayoutView> {
        let mut node = self.node;
        while !node.is_element() {
            match node.parent_node() {
                None => fail!("no nearest element?!"),
                Some(parent) => node = parent,
            }
        }
        node
    }

    /// Always inline for SCCP.
    ///
    /// FIXME(pcwalton): Just replace with the clear type from the style module for speed?
    #[inline(always)]
    pub fn clear(&self) -> Option<ClearType> {
        let style = self.node.style();
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
        let my_style = self.nearest_ancestor_element().style();

        debug!("(font style) start: {:?}", self.nearest_ancestor_element().type_id());

        // FIXME: Too much allocation here.
        let font_families = do my_style.Font.font_family.map |family| {
            match *family {
                font_family::FamilyName(ref name) => (*name).clone(),
            }
        };
        let font_families = font_families.connect(", ");
        debug!("(font style) font families: `{:s}`", font_families);

        let font_size = my_style.Font.font_size.to_f64().unwrap() / 60.0;
        debug!("(font style) font size: `{:f}px`", font_size);

        let (italic, oblique) = match my_style.Font.font_style {
            font_style::normal => (false, false),
            font_style::italic => (true, false),
            font_style::oblique => (false, true),
        };

        FontStyle {
            pt_size: font_size,
            weight: FontWeight300,
            italic: italic,
            oblique: oblique,
            families: font_families,
        }
    }

    // FIXME(pcwalton): Why &'static??? Isn't that wildly unsafe?
    #[inline(always)]
    pub fn style(&self) -> &'static ComputedValues {
        self.node.style()
    }

    /// Returns the text alignment of the computed style of the nearest ancestor-or-self `Element`
    /// node.
    pub fn text_align(&self) -> text_align::T {
        self.nearest_ancestor_element().style().Text.text_align
    }

    pub fn line_height(&self) -> line_height::T {
        self.nearest_ancestor_element().style().Box.line_height
    }

    pub fn vertical_align(&self) -> vertical_align::T {
        self.nearest_ancestor_element().style().Box.vertical_align
    }

    /// Returns the text decoration of the computed style of the nearest `Element` node
    pub fn text_decoration(&self) -> text_decoration::T {
        /// Computes the propagated value of text-decoration, as specified in CSS 2.1 § 16.3.1
        /// TODO: make sure this works with anonymous box generation.
        fn get_propagated_text_decoration(element: AbstractNode<LayoutView>)
                                          -> text_decoration::T {
            //Skip over non-element nodes in the DOM
            if !element.is_element() {
                return match element.parent_node() {
                    None => text_decoration::none,
                    Some(parent) => get_propagated_text_decoration(parent),
                };
            }

            // FIXME: Implement correctly.
            let display_in_flow = true;

            let position = element.style().Box.position;
            let float = element.style().Box.float;

            let in_flow = (position == position::static_) && (float == float::none) &&
                display_in_flow;

            let text_decoration = element.style().Text.text_decoration;

            if text_decoration == text_decoration::none && in_flow {
                match element.parent_node() {
                    None => text_decoration::none,
                    Some(parent) => get_propagated_text_decoration(parent),
                }
            } else {
                text_decoration
            }
        }
        get_propagated_text_decoration(self.nearest_ancestor_element())
    }

    /// Returns the sum of margin, border, and padding on the left.
    pub fn offset(&self) -> Au {
        self.margin.get().left + self.border.get().left + self.padding.get().left
    }

    /// Returns true if this element is replaced content. This is true for images, form elements,
    /// and so on.
    pub fn is_replaced(&self) -> bool {
        match self.specific {
            ImageBox(*) => true,
            _ => false,
        }
    }

    /// Returns true if this element can be split. This is true for text boxes.
    pub fn can_split(&self) -> bool {
        match self.specific {
            ScannedTextBox(*) => true,
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

    /// Adds the display items necessary to paint the background of this box to the display list if
    /// necessary.
    pub fn paint_background_if_applicable<E:ExtraDisplayListData>(
                                          @self,
                                          list: &Cell<DisplayList<E>>,
                                          absolute_bounds: &Rect<Au>) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a box".
        let nearest_ancestor_element = self.nearest_ancestor_element();

        let style = nearest_ancestor_element.style();
        let background_color = style.resolve_color(style.Background.background_color);
        if !background_color.alpha.approx_eq(&0.0) {
            list.with_mut_ref(|list| {
                let solid_color_display_item = ~SolidColorDisplayItem {
                    base: BaseDisplayItem {
                        bounds: *absolute_bounds,
                        extra: ExtraDisplayListData::new(&self),
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
                                       @self,
                                       list: &Cell<DisplayList<E>>,
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

        // Append the border to the display list.
        do list.with_mut_ref |list| {
            let border_display_item = ~BorderDisplayItem {
                base: BaseDisplayItem {
                    bounds: *abs_bounds,
                    extra: ExtraDisplayListData::new(&self),
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
        }
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
                              @self,
                              _: &DisplayListBuilder,
                              dirty: &Rect<Au>,
                              offset: &Point2D<Au>,
                              list: &Cell<DisplayList<E>>) {
        let box_bounds = self.position.get();
        let absolute_box_bounds = box_bounds.translate(offset);
        debug!("Box::build_display_list at rel={}, abs={}: {:s}",
               box_bounds, absolute_box_bounds, self.debug_str());
        debug!("Box::build_display_list: dirty={}, offset={}", *dirty, *offset);

        if self.nearest_ancestor_element().style().Box.visibility != visibility::visible {
            return;
        }

        if absolute_box_bounds.intersects(dirty) {
            debug!("Box::build_display_list: intersected. Adding display item...");
        } else {
            debug!("Box::build_display_list: Did not intersect...");
            return;
        }

        // Add the background to the list, if applicable.
        self.paint_background_if_applicable(list, &absolute_box_bounds);

        match self.specific {
            UnscannedTextBox(_) => fail!("Shouldn't see unscanned boxes here."),
            ScannedTextBox(ref text_box) => {
                do list.with_mut_ref |list| {
                    let item = ~ClipDisplayItem {
                        base: BaseDisplayItem {
                            bounds: absolute_box_bounds,
                            extra: ExtraDisplayListData::new(&self),
                        },
                        child_list: ~[],
                        need_clip: false
                    };
                    list.append_item(ClipDisplayItemClass(item));
                }


                let nearest_ancestor_element = self.nearest_ancestor_element();
                let color = nearest_ancestor_element.style().Color.color.to_gfx_color();

                // Create the text box.
                do list.with_mut_ref |list| {
                    let text_display_item = ~TextDisplayItem {
                        base: BaseDisplayItem {
                            bounds: absolute_box_bounds,
                            extra: ExtraDisplayListData::new(&self),
                        },
                        // FIXME(pcwalton): Allocation? Why?!
                        text_run: ~text_box.run.serialize(),
                        range: text_box.range,
                        color: color,
                    };

                    list.append_item(TextDisplayItemClass(text_display_item))
                }

                // Draw debug frames for text bounds.
                //
                // FIXME(pcwalton): This is a bit of an abuse of the logging infrastructure. We
                // should have a real `SERVO_DEBUG` system.
                debug!("{:?}", {
                    // Compute the text box bounds and draw a border surrounding them.
                    let debug_border = SideOffsets2D::new_all_same(Au::from_px(1));

                    do list.with_mut_ref |list| {
                        let border_display_item = ~BorderDisplayItem {
                            base: BaseDisplayItem {
                                bounds: absolute_box_bounds,
                                extra: ExtraDisplayListData::new(&self),
                            },
                            border: debug_border,
                            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
                            style: SideOffsets2D::new_all_same(border_style::solid)

                        };
                        list.append_item(BorderDisplayItemClass(border_display_item))
                    }

                    // Draw a rectangle representing the baselines.
                    //
                    // TODO(Issue #221): Create and use a Line display item for the baseline.
                    let ascent = text_box.run.metrics_for_range(
                        &text_box.range).ascent;
                    let baseline = Rect(absolute_box_bounds.origin + Point2D(Au(0), ascent),
                                        Size2D(absolute_box_bounds.size.width, Au(0)));

                    do list.with_mut_ref |list| {
                        let border_display_item = ~BorderDisplayItem {
                            base: BaseDisplayItem {
                                bounds: baseline,
                                extra: ExtraDisplayListData::new(&self),
                            },
                            border: debug_border,
                            color: SideOffsets2D::new_all_same(rgb(0, 200, 0)),
                            style: SideOffsets2D::new_all_same(border_style::dashed)

                        };
                        list.append_item(BorderDisplayItemClass(border_display_item))
                    }

                    ()
                });
            },
            GenericBox => {
                do list.with_mut_ref |list| {
                    let item = ~ClipDisplayItem {
                        base: BaseDisplayItem {
                            bounds: absolute_box_bounds,
                            extra: ExtraDisplayListData::new(&self),
                        },
                        child_list: ~[],
                        need_clip: self.needs_clip()
                    };
                    list.append_item(ClipDisplayItemClass(item));
                }

                // FIXME(pcwalton): This is a bit of an abuse of the logging infrastructure. We
                // should have a real `SERVO_DEBUG` system.
                debug!("{:?}", {
                    let debug_border = SideOffsets2D::new_all_same(Au::from_px(1));

                    do list.with_mut_ref |list| {
                        let border_display_item = ~BorderDisplayItem {
                            base: BaseDisplayItem {
                                bounds: absolute_box_bounds,
                                extra: ExtraDisplayListData::new(&self),
                            },
                            border: debug_border,
                            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
                            style: SideOffsets2D::new_all_same(border_style::solid)

                        };
                        list.append_item(BorderDisplayItemClass(border_display_item))
                    }

                    ()
                });
            },
            ImageBox(ref image_box) => {
                do list.with_mut_ref |list| {
                    let item = ~ClipDisplayItem {
                        base: BaseDisplayItem {
                            bounds: absolute_box_bounds,
                            extra: ExtraDisplayListData::new(&self),
                        },
                        child_list: ~[],
                        need_clip: false
                    };
                    list.append_item(ClipDisplayItemClass(item));
                }

                match image_box.image.mutate().ptr.get_image() {
                    Some(image) => {
                        debug!("(building display list) building image box");

                        // Place the image into the display list.
                        do list.with_mut_ref |list| {
                            let image_display_item = ~ImageDisplayItem {
                                base: BaseDisplayItem {
                                    bounds: absolute_box_bounds,
                                    extra: ExtraDisplayListData::new(&self),
                                },
                                image: image.clone(),
                            };
                            list.append_item(ImageDisplayItemClass(image_display_item))
                        }
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

        // Add a border, if applicable.
        //
        // TODO: Outlines.
        self.paint_borders_if_applicable(list, &absolute_box_bounds);
    }

    /// Returns the *minimum width* and *preferred width* of this box as defined by CSS 2.1.
    pub fn minimum_and_preferred_widths(&self) -> (Au, Au) {
        let guessed_width = self.guess_width();
        let (additional_minimum, additional_preferred) = match self.specific {
            GenericBox => (Au(0), Au(0)),
            ImageBox(ref image_box_info) => {
                let image_width = image_box_info.image_width(self);
                (image_width, image_width)
            }
            ScannedTextBox(ref text_box_info) => {
                let range = &text_box_info.range;
                let min_line_width = text_box_info.run.min_width_for_range(range);

                let mut max_line_width = Au::new(0);
                for line_range in text_box_info.run.iter_natural_lines_for_range(range) {
                    let line_metrics = text_box_info.run.metrics_for_range(&line_range);
                    max_line_width = Au::max(max_line_width, line_metrics.advance_width);
                }

                (min_line_width, max_line_width)
            }
            UnscannedTextBox(*) => fail!("Unscanned text boxes should have been scanned by now!"),
        };
        (guessed_width + additional_minimum, guessed_width + additional_preferred)
    }

    /// Returns, and computes, the height of this box.
    ///
    /// FIXME(pcwalton): Rename to just `height`?
    /// FIXME(pcwalton): This function *mutates* the height? Gross! Refactor please.
    pub fn box_height(&self) -> Au {
        match self.specific {
            GenericBox => Au(0),
            ImageBox(ref image_box_info) => {
                let size = image_box_info.image.mutate().ptr.get_size();
                let height = Au::from_px(size.unwrap_or(Size2D(0, 0)).height);

                // Eww. Refactor this.
                self.position.mutate().ptr.size.height = height;
                debug!("box_height: found image height: {}", height);

                height
            }
            ScannedTextBox(ref text_box_info) => {
                // Compute the height based on the line-height and font size.
                let (range, run) = (&text_box_info.range, &text_box_info.run);
                let text_bounds = run.metrics_for_range(range).bounding_box;
                let em_size = text_bounds.size.height;
                self.calculate_line_height(em_size)
            }
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
        }
    }

    /// Attempts to split this box so that its width is no more than `max_width`.
    pub fn split_to_width(@self, max_width: Au, starts_line: bool) -> SplitBoxResult {
        match self.specific {
            GenericBox | ImageBox(_) => CannotSplit(self),
            UnscannedTextBox(_) => fail!("Unscanned text boxes should have been scanned by now!"),
            ScannedTextBox(ref text_box_info) => {
                let mut pieces_processed_count: uint = 0;
                let mut remaining_width: Au = max_width;
                let mut left_range = Range::new(text_box_info.range.begin(), 0);
                let mut right_range: Option<Range> = None;

                debug!("split_to_width: splitting text box (strlen={:u}, range={}, \
                                                            avail_width={})",
                       text_box_info.run.text.get().len(),
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
                    let new_text_box_info = ScannedTextBoxInfo::new(text_box_info.run, left_range);
                    let new_text_box = @Box::new(self.node, ScannedTextBox(new_text_box_info));
                    let new_metrics = new_text_box_info.run.metrics_for_range(&left_range);
                    new_text_box.set_size(new_metrics.bounding_box.size);
                    Some(new_text_box)
                } else {
                    None
                };

                let right_box = right_range.map_default(None, |range: Range| {
                    let new_text_box_info = ScannedTextBoxInfo::new(text_box_info.run, range);
                    let new_text_box = @Box::new(self.node, ScannedTextBox(new_text_box_info));
                    let new_metrics = new_text_box_info.run.metrics_for_range(&range);
                    new_text_box.set_size(new_metrics.bounding_box.size);
                    Some(new_text_box)
                });

                if pieces_processed_count == 1 || left_box.is_none() {
                    SplitDidNotFit(left_box, right_box)
                } else {
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
    pub fn assign_width(&self) {
        match self.specific {
            GenericBox => {
                // FIXME(pcwalton): This seems clownshoes; can we remove?
                self.position.mutate().ptr.size.width = Au::from_px(45)
            }
            ImageBox(ref image_box_info) => {
                let image_width = image_box_info.image_width(self);
                self.position.mutate().ptr.size.width = image_width
            }
            ScannedTextBox(_) => {
                // Scanned text boxes will have already had their widths assigned by this point.
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
            ScannedTextBox(ref text_box_info) => text_box_info.run.teardown(),
            _ => {}
        }
    }

    /// Returns true if the contents should be clipped (i.e. if `overflow` is `hidden`).
    pub fn needs_clip(&self) -> bool {
        self.node.style().Box.overflow == overflow::hidden
    }

    /// Returns a debugging string describing this box.
    ///
    /// TODO(pcwalton): Reimplement.
    pub fn debug_str(&self) -> ~str {
        ~"(Box)"
    }
}

