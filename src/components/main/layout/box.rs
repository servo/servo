/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `RenderBox` type, which represents the leaves of the layout tree.

use css::node_style::StyledNode;
use dom::node::AbstractNode;
use layout::context::LayoutContext;
use layout::debug::DebugMethods;
use layout::display_list_builder::{DisplayListBuilder, ToGfxColor};
use layout::flow::FlowContext;
use layout::text;

use core::cell::Cell;
use core::cmp::ApproxEq;
use core::managed;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::{DisplayItem, DisplayList};
use gfx::font::{FontStyle, FontWeight300};
use gfx::geometry::Au;
use gfx::text::text_run::TextRun;
use newcss::color::rgb;
use newcss::complete::CompleteStyle;
use newcss::units::{Cursive, Em, Fantasy, Monospace, Pt, Px, SansSerif, Serif};
use newcss::values::{CSSBorderWidthLength, CSSBorderWidthMedium};
use newcss::values::{CSSFontFamilyFamilyName, CSSFontFamilyGenericFamily};
use newcss::values::{CSSFontSizeLength, CSSFontStyleItalic, CSSFontStyleNormal};
use newcss::values::{CSSFontStyleOblique, CSSTextAlign, CSSTextDecoration};
use newcss::values::{CSSTextDecorationNone, CSSFloatNone, CSSPositionStatic};
use newcss::values::{CSSDisplayInlineBlock, CSSDisplayInlineTable};
use servo_net::image::holder::ImageHolder;
use servo_net::local_image_cache::LocalImageCache;
use servo_util::range::*;
use std::arc;
use std::net::url::Url;

/// Render boxes (`struct RenderBox`) are the leaves of the layout tree. They cannot position
/// themselves. In general, render boxes do not have a simple correspondence with CSS boxes as in
/// the specification:
///
/// * Several render boxes may correspond to the same CSS box or DOM node. For example, a CSS text
///   box broken across two lines is represented by two render boxes.
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
pub enum RenderBox {
    GenericRenderBoxClass(@mut RenderBoxBase),
    ImageRenderBoxClass(@mut ImageRenderBox),
    TextRenderBoxClass(@mut TextRenderBox),
    UnscannedTextRenderBoxClass(@mut UnscannedTextRenderBox),
}

impl RenderBox {
    pub fn teardown(&self) {
        match *self {
            TextRenderBoxClass(box) => box.teardown(),
            _ => ()
        }
    }
}

/// A box that represents a (replaced content) image and its accompanying borders, shadows, etc.
pub struct ImageRenderBox {
    base: RenderBoxBase,
    image: ImageHolder,
}

impl ImageRenderBox {
    pub fn new(base: RenderBoxBase, image_url: Url, local_image_cache: @mut LocalImageCache)
               -> ImageRenderBox {
        assert!(base.node.is_image_element());

        ImageRenderBox {
            base: base,
            image: ImageHolder::new(image_url, local_image_cache),
        }
    }
}

/// A box representing a single run of text with a distinct style. A `TextRenderBox` may be split
/// into two or more render boxes across line breaks. Several `TextBox`es may correspond to a
/// single DOM text node. Split text boxes are implemented by referring to subsets of a master
/// `TextRun` object.
pub struct TextRenderBox {
    base: RenderBoxBase,
    run: @TextRun,
    range: Range,
}

impl TextRenderBox {
    fn teardown(&self) {
        self.run.teardown();
    }
}

/// The data for an unscanned text box.
pub struct UnscannedTextRenderBox {
    base: RenderBoxBase,
    text: ~str,
}

impl UnscannedTextRenderBox {
    /// Creates a new instance of `UnscannedTextRenderBox`.
    pub fn new(base: RenderBoxBase) -> UnscannedTextRenderBox {
        assert!(base.node.is_text());

        do base.node.with_imm_text |text_node| {
            // FIXME: Don't copy text; atomically reference count it instead.
            // FIXME(pcwalton): If we're just looking at node data, do we have to ensure this is
            // a text node?
            UnscannedTextRenderBox {
                base: base,
                text: text_node.parent.data.to_str(),
            }
        }
    }
}

pub enum RenderBoxType {
    RenderBox_Generic,
    RenderBox_Image,
    RenderBox_Text,
}

/// Represents the outcome of attempting to split a render box.
pub enum SplitBoxResult {
    CannotSplit(RenderBox),
    // in general, when splitting the left or right side can
    // be zero length, due to leading/trailing trimmable whitespace
    SplitDidFit(Option<RenderBox>, Option<RenderBox>),
    SplitDidNotFit(Option<RenderBox>, Option<RenderBox>)
}

/// Data common to all render boxes.
pub struct RenderBoxBase {
    /// The DOM node that this `RenderBox` originates from.
    node: AbstractNode,

    /// The reference to the containing flow context which this box participates in.
    ctx: FlowContext,

    /// The position of this box relative to its owning flow.
    position: Rect<Au>,

    /// A debug ID.
    ///
    /// TODO(#87) Make this only present in debug builds.
    id: int
}

impl RenderBoxBase {
    /// Constructs a new `RenderBoxBase` instance.
    pub fn new(node: AbstractNode, flow_context: FlowContext, id: int) -> RenderBoxBase {
        RenderBoxBase {
            node: node,
            ctx: flow_context,
            position: Au::zero_rect(),
            id: id,
        }
    }
}

pub impl RenderBox {
    /// Borrows this render box immutably in order to work with its common data.
    #[inline(always)]
    fn with_imm_base<R>(&self, callback: &fn(&RenderBoxBase) -> R) -> R {
        match *self {
            GenericRenderBoxClass(generic_box) => callback(generic_box),
            ImageRenderBoxClass(image_box) => {
                callback(&image_box.base)
            }
            TextRenderBoxClass(text_box) => {
                callback(&text_box.base)
            }
            UnscannedTextRenderBoxClass(unscanned_text_box) => {
                callback(&unscanned_text_box.base)
            }
        }
    }

    /// Borrows this render box mutably in order to work with its common data.
    #[inline(always)]
    fn with_mut_base<R>(&self, callback: &fn(&mut RenderBoxBase) -> R) -> R {
        match *self {
            GenericRenderBoxClass(generic_box) => callback(generic_box),
            ImageRenderBoxClass(image_box) => {
                callback(&mut image_box.base)
            }
            TextRenderBoxClass(text_box) => {
                callback(&mut text_box.base)
            }
            UnscannedTextRenderBoxClass(unscanned_text_box) => {
                callback(&mut unscanned_text_box.base)
            }
        }
    }

    /// A convenience function to return the position of this box.
    fn position(&self) -> Rect<Au> {
        do self.with_imm_base |base| {
            base.position
        }
    }

    /// A convenience function to return the debugging ID of this box.
    fn id(&self) -> int {
        do self.with_mut_base |base| {
            base.id
        }
    }

    /// Returns true if this element is replaced content. This is true for images, form elements,
    /// and so on.
    fn is_replaced(&self) -> bool {
        match *self {
            ImageRenderBoxClass(*) => true,
            _ => false
        }
    }

    /// Returns true if this element can be split. This is true for text boxes.
    fn can_split(&self) -> bool {
        match *self {
            TextRenderBoxClass(*) => true,
            _ => false
        }
    }

    /// Returns true if this element is an unscanned text box that consists entirely of whitespace.
    fn is_whitespace_only(&self) -> bool {
        match *self {
            UnscannedTextRenderBoxClass(unscanned_text_box) => {
                unscanned_text_box.text.is_whitespace()
            }
            _ => false
        }
    }

    /// Determines whether this box can merge with another render box.
    fn can_merge_with_box(&self, other: RenderBox) -> bool {
        match (self, &other) {
            (&UnscannedTextRenderBoxClass(*), &UnscannedTextRenderBoxClass(*)) => {
                self.font_style() == other.font_style() && self.text_decoration() == other.text_decoration()
            },
            (&TextRenderBoxClass(text_box_a), &TextRenderBoxClass(text_box_b)) => {
                managed::ptr_eq(text_box_a.run, text_box_b.run)
            }
            (_, _) => false,
        }
    }

    /// Attempts to split this box so that its width is no more than `max_width`. Fails if this box
    /// is an unscanned text box.
    fn split_to_width(&self, _: &LayoutContext, max_width: Au, starts_line: bool)
                      -> SplitBoxResult {
        match *self {
            GenericRenderBoxClass(*) | ImageRenderBoxClass(*) => CannotSplit(*self),
            UnscannedTextRenderBoxClass(*) => {
                fail!(~"WAT: shouldn't be an unscanned text box here.")
            }

            TextRenderBoxClass(text_box) => {
                let mut pieces_processed_count: uint = 0;
                let mut remaining_width: Au = max_width;
                let mut left_range = Range::new(text_box.range.begin(), 0);
                let mut right_range: Option<Range> = None;

                debug!("split_to_width: splitting text box (strlen=%u, range=%?, avail_width=%?)",
                       text_box.run.text.len(),
                       text_box.range,
                       max_width);

                for text_box.run.iter_indivisible_pieces_for_range(
                        &text_box.range) |piece_range| {
                    debug!("split_to_width: considering piece (range=%?, remain_width=%?)",
                           piece_range,
                           remaining_width);

                    let metrics = text_box.run.metrics_for_range(piece_range);
                    let advance = metrics.advance_width;
                    let should_continue: bool;

                    if advance <= remaining_width {
                        should_continue = true;

                        if starts_line &&
                                pieces_processed_count == 0 &&
                                text_box.run.range_is_trimmable_whitespace(piece_range) {
                            debug!("split_to_width: case=skipping leading trimmable whitespace");
                            left_range.shift_by(piece_range.length() as int);
                        } else {
                            debug!("split_to_width: case=enlarging span");
                            remaining_width -= advance;
                            left_range.extend_by(piece_range.length() as int);
                        }
                    } else {    // The advance is more than the remaining width.
                        should_continue = false;

                        if text_box.run.range_is_trimmable_whitespace(piece_range) {
                            // If there are still things after the trimmable whitespace, create the
                            // right chunk.
                            if piece_range.end() < text_box.range.end() {
                                debug!("split_to_width: case=skipping trimmable trailing \
                                        whitespace, then split remainder");
                                let right_range_end =
                                    text_box.range.end() - piece_range.end();
                                right_range = Some(Range::new(piece_range.end(), right_range_end));
                            } else {
                                debug!("split_to_width: case=skipping trimmable trailing \
                                        whitespace");
                            }
                        } else if piece_range.begin() < text_box.range.end() {
                            // There are still some things left over at the end of the line. Create
                            // the right chunk.
                            let right_range_end =
                                text_box.range.end() - piece_range.begin();
                            right_range = Some(Range::new(piece_range.begin(), right_range_end));
                            debug!("split_to_width: case=splitting remainder with right range=%?",
                                   right_range);
                        }
                    }

                    pieces_processed_count += 1;

                    if !should_continue {
                        break
                    }
                }

                let left_box = if left_range.length() > 0 {
                    let new_text_box = @mut text::adapt_textbox_with_range(text_box.base,
                                                                           text_box.run,
                                                                           left_range);
                    Some(TextRenderBoxClass(new_text_box))
                } else {
                    None
                };

                let right_box = do right_range.map_default(None) |range: &Range| {
                    let new_text_box = @mut text::adapt_textbox_with_range(text_box.base,
                                                                           text_box.run,
                                                                           *range);
                    Some(TextRenderBoxClass(new_text_box))
                };

                if pieces_processed_count == 1 || left_box.is_none() {
                    SplitDidNotFit(left_box, right_box)
                } else {
                    SplitDidFit(left_box, right_box)
                }
            }
        }
    }

    /// Returns the *minimum width* of this render box as defined by the CSS specification.
    fn get_min_width(&self, _: &LayoutContext) -> Au {
        match *self {
            // TODO: This should account for the minimum width of the box element in isolation.
            // That includes borders, margins, and padding, but not child widths. The block
            // `FlowContext` will combine the width of this element and that of its children to
            // arrive at the context width.
            GenericRenderBoxClass(*) => Au(0),

            ImageRenderBoxClass(image_box) => {
                // TODO: Consult the CSS `width` property as well as margins and borders.
                // TODO: If the image isn't available, consult `width`.
                Au::from_px(image_box.image.get_size().get_or_default(Size2D(0, 0)).width)
            }

            TextRenderBoxClass(text_box) => {
                text_box.run.min_width_for_range(&text_box.range)
            }

            UnscannedTextRenderBoxClass(*) => fail!(~"Shouldn't see unscanned boxes here.")
        }
    }

    /// Returns the *preferred width* of this render box as defined by the CSS specification.
    fn get_pref_width(&self, _: &LayoutContext) -> Au {
        match *self {
            // TODO: This should account for the preferred width of the box element in isolation.
            // That includes borders, margins, and padding, but not child widths. The block
            // `FlowContext` will combine the width of this element and that of its children to
            // arrive at the context width.
            GenericRenderBoxClass(*) => Au(0),

            ImageRenderBoxClass(image_box) => {
                Au::from_px(image_box.image.get_size().get_or_default(Size2D(0, 0)).width)
            }

            TextRenderBoxClass(text_box) => {
                // A text box cannot span lines, so assume that this is an unsplit text box.
                //
                // TODO: If text boxes have been split to wrap lines, then they could report a
                // smaller preferred width during incremental reflow. Maybe text boxes should
                // report nothing and the parent flow can factor in minimum/preferred widths of any
                // text runs that it owns.
                let mut max_line_width = Au(0);
                for text_box.run.iter_natural_lines_for_range(&text_box.range)
                        |line_range| {
                    let mut line_width: Au = Au(0);
                    for text_box.run.glyphs.iter_glyphs_for_char_range(line_range)
                            |_, glyph| {
                        line_width += glyph.advance()
                    }

                    max_line_width = Au::max(max_line_width, line_width);
                }

                max_line_width
            }

            UnscannedTextRenderBoxClass(*) => fail!(~"Shouldn't see unscanned boxes here."),
        }
    }

    /// Returns the amount of left and right "fringe" used by this box. This is based on margins,
    /// borders, padding, and width.
    fn get_used_width(&self) -> (Au, Au) {
        // TODO: This should actually do some computation! See CSS 2.1, Sections 10.3 and 10.4.
        (Au(0), Au(0))
    }

    /// Returns the amount of left and right "fringe" used by this box. This should be based on
    /// margins, borders, padding, and width.
    fn get_used_height(&self) -> (Au, Au) {
        // TODO: This should actually do some computation! See CSS 2.1, Sections 10.5 and 10.6.
        (Au(0), Au(0))
    }

    /// The box formed by the content edge as defined in CSS 2.1 § 8.1. Coordinates are relative to
    /// the owning flow.
    fn content_box(&self) -> Rect<Au> {
        let origin = self.position().origin;
        match *self {
            ImageRenderBoxClass(image_box) => {
                Rect {
                    origin: origin,
                    size: image_box.base.position.size,
                }
            },
            GenericRenderBoxClass(*) => {
                self.position()

                // FIXME: The following hits an ICE for whatever reason.

                /*
                let origin = self.d().position.origin;
                let size  = self.d().position.size;
                let (offset_left, offset_right) = self.get_used_width();
                let (offset_top, offset_bottom) = self.get_used_height();

                Rect {
                    origin: Point2D(origin.x + offset_left, origin.y + offset_top),
                    size: Size2D(size.width - (offset_left + offset_right),
                                 size.height - (offset_top + offset_bottom))
                }
                */
            },
            TextRenderBoxClass(*) => self.position(),
            UnscannedTextRenderBoxClass(*) => fail!(~"Shouldn't see unscanned boxes here.")
        }
    }

    /// The box formed by the border edge as defined in CSS 2.1 § 8.1. Coordinates are relative to
    /// the owning flow.
    fn border_box(&self) -> Rect<Au> {
        // TODO: Actually compute the content box, padding, and border.
        self.content_box()
    }

    /// The box formed by the margin edge as defined in CSS 2.1 § 8.1. Coordinates are relative to
    /// the owning flow.
    fn margin_box(&self) -> Rect<Au> {
        // TODO: Actually compute the content_box, padding, border, and margin.
        self.content_box()
    }

    /// A convenience function to determine whether this render box represents a DOM element.
    fn is_element(&self) -> bool {
        do self.with_imm_base |base| {
            base.node.is_element()
        }
    }

    /// A convenience function to access the computed style of the DOM node that this render box
    /// represents.
    fn style(&self) -> CompleteStyle {
        self.with_imm_base(|base| base.node.style())
    }

    /// A convenience function to access the DOM node that this render box represents.
    fn node(&self) -> AbstractNode {
        self.with_imm_base(|base| base.node)
    }

    /// Returns the nearest ancestor-or-self `Element` to the DOM node that this render box
    /// represents.
    ///
    /// If there is no ancestor-or-self `Element` node, fails.
    fn nearest_ancestor_element(&self) -> AbstractNode {
        do self.with_imm_base |base| {
            let mut node = base.node;
            while !node.is_element() {
                match node.parent_node() {
                    None => fail!(~"no nearest element?!"),
                    Some(parent) => node = parent,
                }
            }
            node
        }
    }

    //
    // Painting
    //

    /// Adds the display items for this render box to the given display list.
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
    fn build_display_list(&self,
                          _: &DisplayListBuilder,
                          dirty: &Rect<Au>,
                          offset: &Point2D<Au>,
                          list: &Cell<DisplayList>) {
        let box_bounds = self.position();
        let absolute_box_bounds = box_bounds.translate(offset);
        debug!("RenderBox::build_display_list at rel=%?, abs=%?: %s",
               box_bounds, absolute_box_bounds, self.debug_str());
        debug!("RenderBox::build_display_list: dirty=%?, offset=%?", dirty, offset);

        if absolute_box_bounds.intersects(dirty) {
            debug!("RenderBox::build_display_list: intersected. Adding display item...");
        } else {
            debug!("RenderBox::build_display_list: Did not intersect...");
            return;
        }

        // Add the background to the list, if applicable.
        self.paint_background_if_applicable(list, &absolute_box_bounds);

        match *self {
            UnscannedTextRenderBoxClass(*) => fail!(~"Shouldn't see unscanned boxes here."),
            TextRenderBoxClass(text_box) => {
                let nearest_ancestor_element = self.nearest_ancestor_element();
                let color = nearest_ancestor_element.style().color().to_gfx_color();

                // FIXME: This should use `with_mut_ref` when that appears.
                let mut this_list = list.take();
                this_list.append_item(~DisplayItem::new_Text(&absolute_box_bounds,
                                                             ~text_box.run.serialize(),
                                                             text_box.range,
                                                             color));
                list.put_back(this_list);

                // Draw debug frames for text bounds.
                //
                // FIXME(pcwalton): This is a bit of an abuse of the logging infrastructure. We
                // should have a real `SERVO_DEBUG` system.
                debug!("%?", {
                    // Compute the text box bounds.
                    //
                    // FIXME: This should use `with_mut_ref` when that appears.
                    let mut this_list = list.take();
                    this_list.append_item(~DisplayItem::new_Border(&absolute_box_bounds,
                                                                   Au::from_px(1),
                                                                   rgb(0, 0, 200).to_gfx_color()));

                    // Draw a rectangle representing the baselines.
                    //
                    // TODO(Issue #221): Create and use a Line display item for the baseline.
                    let ascent = text_box.run.metrics_for_range(
                        &text_box.range).ascent;
                    let baseline = Rect(absolute_box_bounds.origin + Point2D(Au(0), ascent),
                                        Size2D(absolute_box_bounds.size.width, Au(0)));

                    this_list.append_item(~DisplayItem::new_Border(&baseline,
                                                                   Au::from_px(1),
                                                                   rgb(0, 200, 0).to_gfx_color()));
                    list.put_back(this_list);
                    ()
                });
            },

            GenericRenderBoxClass(_) => {}

            ImageRenderBoxClass(image_box) => {
                match image_box.image.get_image() {
                    Some(image) => {
                        debug!("(building display list) building image box");

                        // FIXME: This should use `with_mut_ref` when that appears.
                        let mut this_list = list.take();
                        this_list.append_item(~DisplayItem::new_Image(&absolute_box_bounds,
                                                                      arc::clone(&image)));
                        list.put_back(this_list);
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

    /// Adds the display items necessary to paint the background of this render box to the display
    /// list if necessary.
    fn paint_background_if_applicable(&self,
                                      list: &Cell<DisplayList>,
                                      absolute_bounds: &Rect<Au>) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a render box".
        let nearest_ancestor_element = self.nearest_ancestor_element();

        let bgcolor = nearest_ancestor_element.style().background_color();
        if !bgcolor.alpha.approx_eq(&0.0) {
            let mut l = list.take(); // FIXME: use with_mut_ref when available
            l.append_item(~DisplayItem::new_SolidColor(absolute_bounds, bgcolor.to_gfx_color()));
            list.put_back(l);
        }
    }

    /// Adds the display items necessary to paint the borders of this render box to the display
    /// list if necessary.
    fn paint_borders_if_applicable(&self, list: &Cell<DisplayList>, abs_bounds: &Rect<Au>) {
        if !self.is_element() {
            return
        }

        let style = self.style();
        let (top_width, right_width) = (style.border_top_width(), style.border_right_width());
        let (bottom_width, left_width) = (style.border_bottom_width(), style.border_left_width());
        match (top_width, right_width, bottom_width, left_width) {
            (CSSBorderWidthLength(Px(top)),
             CSSBorderWidthLength(Px(right)),
             CSSBorderWidthLength(Px(bottom)),
             CSSBorderWidthLength(Px(left))) => {
                let top_au = Au::from_frac_px(top);
                let right_au = Au::from_frac_px(right);
                let bottom_au = Au::from_frac_px(bottom);
                let left_au = Au::from_frac_px(left);

                // Are all the widths equal?
                if [ top_au, right_au, bottom_au ].all(|a| *a == left_au) {
                    let border_width = top_au;
                    let bounds = Rect {
                        origin: Point2D {
                            x: abs_bounds.origin.x - border_width / Au(2),
                            y: abs_bounds.origin.y - border_width / Au(2),
                        },
                        size: Size2D {
                            width: abs_bounds.size.width + border_width,
                            height: abs_bounds.size.height + border_width
                        }
                    };

                    let top_color = self.style().border_top_color();
                    let color = top_color.to_gfx_color(); // FIXME

                    // FIXME: Use `with_mut_ref` when that works.
                    let mut this_list = list.take();
                    this_list.append_item(~DisplayItem::new_Border(&bounds, border_width, color));
                    list.put_back(this_list);
                } else {
                    warn!("ignoring unimplemented border widths");
                }
            }
            (CSSBorderWidthMedium,
             CSSBorderWidthMedium,
             CSSBorderWidthMedium,
             CSSBorderWidthMedium) => {
                // FIXME: This seems to be the default for non-root nodes. For now we'll ignore it.
                warn!("ignoring medium border widths");
            }
            _ => warn!("ignoring unimplemented border widths")
        }
    }

    /// Converts this node's computed style to a font style used for rendering.
    fn font_style(&self) -> FontStyle {
        let my_style = self.nearest_ancestor_element().style();

        // FIXME: Too much allocation here.
        let font_families = do my_style.font_family().map |family| {
            match *family {
                CSSFontFamilyFamilyName(ref family_str) => copy *family_str,
                CSSFontFamilyGenericFamily(Serif)       => ~"serif",
                CSSFontFamilyGenericFamily(SansSerif)   => ~"sans-serif",
                CSSFontFamilyGenericFamily(Cursive)     => ~"cursive",
                CSSFontFamilyGenericFamily(Fantasy)     => ~"fantasy",
                CSSFontFamilyGenericFamily(Monospace)   => ~"monospace",
            }
        };
        let font_families = str::connect(font_families, ~", ");
        debug!("(font style) font families: `%s`", font_families);

        let font_size = match my_style.font_size() {
            CSSFontSizeLength(Px(length)) |
            CSSFontSizeLength(Pt(length)) |
            CSSFontSizeLength(Em(length)) => length,
            _ => 16.0
        };
        debug!("(font style) font size: `%f`", font_size);

        let (italic, oblique) = match my_style.font_style() {
            CSSFontStyleNormal => (false, false),
            CSSFontStyleItalic => (true, false),
            CSSFontStyleOblique => (false, true),
        };

        FontStyle {
            pt_size: font_size,
            weight: FontWeight300,
            italic: italic,
            oblique: oblique,
            families: font_families,
        }
    }

    /// Returns the text alignment of the computed style of the nearest ancestor-or-self `Element`
    /// node.
    fn text_align(&self) -> CSSTextAlign {
        self.nearest_ancestor_element().style().text_align()
    }

    /// Returns the text decoration of the computed style of the nearest `Element` node
    fn text_decoration(&self) -> CSSTextDecoration {
        /// Computes the propagated value of text-decoration, as specified in CSS 2.1 § 16.3.1
        /// TODO: make sure this works with anonymous box generation.
        fn get_propagated_text_decoration(element: AbstractNode) -> CSSTextDecoration {
            //Skip over non-element nodes in the DOM
            if(!element.is_element()){
                return match element.parent_node() {
                    None => CSSTextDecorationNone,
                    Some(parent) => get_propagated_text_decoration(parent),
                };
            }

            //FIXME: is the root param on display() important?
            let display_in_flow = match element.style().display(false) {
                CSSDisplayInlineTable | CSSDisplayInlineBlock => false,
                _ => true,
            };

            let position = element.style().position();
            let float = element.style().float();

            let in_flow = (position == CSSPositionStatic) && (float == CSSFloatNone) &&
                display_in_flow;

            let text_decoration = element.style().text_decoration();

            if(text_decoration == CSSTextDecorationNone && in_flow){
                match element.parent_node() {
                    None => CSSTextDecorationNone,
                    Some(parent) => get_propagated_text_decoration(parent),
                }
            }
            else {
                text_decoration
            }
        }
        get_propagated_text_decoration(self.nearest_ancestor_element())
    }
}

impl DebugMethods for RenderBox {
    fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps a render box for debugging, with indentation.
    fn dump_indent(&self, indent: uint) {
        let mut string = ~"";
        for uint::range(0u, indent) |_i| {
            string += ~"    ";
        }

        string += self.debug_str();
        debug!("%s", string);
    }

    /// Returns a debugging string describing this box.
    fn debug_str(&self) -> ~str {
        let representation = match *self {
            GenericRenderBoxClass(*) => ~"GenericRenderBox",
            ImageRenderBoxClass(*) => ~"ImageRenderBox",
            TextRenderBoxClass(text_box) => {
                fmt!("TextRenderBox(text=%s)", str::substr(text_box.run.text,
                                                           text_box.range.begin(),
                                                           text_box.range.length()))
            }
            UnscannedTextRenderBoxClass(text_box) => {
                fmt!("UnscannedTextRenderBox(%s)", text_box.text)
            }
        };

        fmt!("box b%?: %s", self.id(), representation)
    }
}

