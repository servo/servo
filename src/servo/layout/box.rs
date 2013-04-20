/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/* Fundamental layout structures and algorithms. */

use css::node_style::StyledNode;
use dom::node::AbstractNode;
use layout::context::LayoutContext;
use layout::debug::BoxedMutDebugMethods;
use layout::display_list_builder::DisplayListBuilder;
use layout::flow::FlowContext;
use layout::text::TextBoxData;
use layout;
use newcss::color::{Color, rgb};
use newcss::complete::CompleteStyle;
use newcss::units::{Cursive, Em, Fantasy, Length, Monospace, Pt, Px, SansSerif, Serif};
use newcss::values::{CSSBorderWidthLength, CSSBorderWidthMedium};
use newcss::values::{CSSFontFamilyFamilyName, CSSFontFamilyGenericFamily};
use newcss::values::{CSSFontSizeLength, CSSFontStyleItalic, CSSFontStyleNormal};
use newcss::values::{CSSFontStyleOblique, CSSTextAlign};

use core::managed;
use core::cell::Cell;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::{DisplayItem, DisplayList};
use gfx::font::{FontStyle, FontWeight300};
use gfx::geometry::Au;
use gfx::image::holder::ImageHolder;
use servo_util::range::*;
use gfx;
use std::arc;

/** 
Render boxes (`struct RenderBox`) are the leafs of the layout
tree. They cannot position themselves. In general, render boxes do not
have a simple correspondence with CSS boxes as in the specification:

 * Several render boxes may correspond to the same CSS box or DOM
   node. For example, a CSS text box broken across two lines is
   represented by two render boxes.

 * Some CSS boxes are not created at all, such as some anonymous block
   boxes induced by inline boxes with block-level sibling boxes. In
   that case, Servo uses an InlineFlow with BlockFlow siblings; the
   InlineFlow is block-level, but not a block container. It is
   positioned as if it were a block box, but its children are
   positioned according to inline flow.

Fundamental box types include:

 * GenericBox: an empty box that contributes only borders, margins,
padding, backgrounds. It is analogous to a CSS nonreplaced content box.

 * ImageBox: a box that represents a (replaced content) image and its
   accompanying borders, shadows, etc.

 * TextBox: a box representing a single run of text with a distinct
   style. A TextBox may be split into two or more render boxes across
   line breaks. Several TextBoxes may correspond to a single DOM text
   node. Split text boxes are implemented by referring to subsets of a
   master TextRun object.

*/

/* A box's kind influences how its styles are interpreted during
   layout.  For example, replaced content such as images are resized
   differently than tables, text, or other content.

   It also holds data specific to different box types, such as text.
*/
pub struct RenderBoxData {
    /* originating DOM node */
    node : AbstractNode,
    /* reference to containing flow context, which this box
       participates in */
    ctx  : @mut FlowContext,
    /* position of this box relative to owning flow */
    position : Rect<Au>,
    font_size : Length,
    /* TODO (Issue #87): debug only */
    id: int
}

pub enum RenderBoxType {
    RenderBox_Generic,
    RenderBox_Image,
    RenderBox_Text,
}

pub enum RenderBox {
    GenericBox(RenderBoxData),
    ImageBox(RenderBoxData, ImageHolder),
    TextBox(RenderBoxData, TextBoxData),
    UnscannedTextBox(RenderBoxData, ~str)
}

pub enum SplitBoxResult {
    CannotSplit(@mut RenderBox),
    // in general, when splitting the left or right side can
    // be zero length, due to leading/trailing trimmable whitespace
    SplitDidFit(Option<@mut RenderBox>, Option<@mut RenderBox>),
    SplitDidNotFit(Option<@mut RenderBox>, Option<@mut RenderBox>)
}

pub fn RenderBoxData(node: AbstractNode, ctx: @mut FlowContext, id: int) -> RenderBoxData {
    RenderBoxData {
        node : node,
        ctx  : ctx,
        position : Au::zero_rect(),
        font_size: Px(0.0),
        id : id
    }
}

impl<'self> RenderBox {
    fn d(&'self mut self) -> &'self mut RenderBoxData {
      unsafe {
        //Rust #5074 - we can't take mutable references to the
        //             data that needs to be returned right now.
        match self {
            &GenericBox(ref d)  => cast::transmute(d),
            &ImageBox(ref d, _) => cast::transmute(d),
            &TextBox(ref d, _)  => cast::transmute(d),
            &UnscannedTextBox(ref d, _) => cast::transmute(d),
        }
      }
    }

    fn is_replaced(self) -> bool {
        match self {
           ImageBox(*) => true, // TODO: form elements, etc
            _ => false
        }
    }

    fn can_split(&self) -> bool {
        match *self {
            TextBox(*) => true,
            _ => false
        }
    }

    fn is_whitespace_only(&self) -> bool {
        match *self {
            UnscannedTextBox(_, ref raw_text) => raw_text.is_whitespace(),
            _ => false
        }
    }

    fn can_merge_with_box(@mut self, other: @mut RenderBox) -> bool {
        assert!(!managed::mut_ptr_eq(self, other));

        match (self, other) {
            (@UnscannedTextBox(*), @UnscannedTextBox(*)) => {
                self.font_style() == other.font_style()
            },
            (@TextBox(_, d1), @TextBox(_, d2)) => managed::ptr_eq(d1.run, d2.run),
            (_, _) => false
        }
    }

    fn split_to_width(@mut self, _ctx: &LayoutContext, max_width: Au, starts_line: bool) -> SplitBoxResult {
        match self {
            @GenericBox(*) => CannotSplit(self),
            @ImageBox(*) => CannotSplit(self),
            @UnscannedTextBox(*) => fail!(~"WAT: shouldn't be an unscanned text box here."),
            @TextBox(_,data) => {

                let mut pieces_processed_count : uint = 0;
                let mut remaining_width : Au = max_width;
                let mut left_range = Range::new(data.range.begin(), 0);
                let mut right_range : Option<Range> = None;
                debug!("split_to_width: splitting text box (strlen=%u, range=%?, avail_width=%?)",
                       data.run.text.len(), data.range, max_width);
                do data.run.iter_indivisible_pieces_for_range(&data.range) |piece_range| {
                    debug!("split_to_width: considering piece (range=%?, remain_width=%?)",
                           piece_range, remaining_width);
                    let metrics = data.run.metrics_for_range(piece_range);
                    let advance = metrics.advance_width;
                    let should_continue : bool;

                    if advance <= remaining_width {
                        should_continue = true;
                        if starts_line && pieces_processed_count == 0 
                            && data.run.range_is_trimmable_whitespace(piece_range) {
                            debug!("split_to_width: case=skipping leading trimmable whitespace");
                            left_range.shift_by(piece_range.length() as int); 
                        } else {
                            debug!("split_to_width: case=enlarging span");
                            remaining_width -= advance;
                            left_range.extend_by(piece_range.length() as int);
                        }
                    } else { /* advance > remaining_width */
                        should_continue = false;

                        if data.run.range_is_trimmable_whitespace(piece_range) {
                            // if there are still things after the trimmable whitespace, create right chunk
                            if piece_range.end() < data.range.end() {
                                debug!("split_to_width: case=skipping trimmable trailing whitespace, then split remainder");
                                right_range = Some(Range::new(piece_range.end(),
                                                              data.range.end() - piece_range.end()));
                            } else {
                                debug!("split_to_width: case=skipping trimmable trailing whitespace");
                            }
                        } else if piece_range.begin() < data.range.end() {
                            // still things left, create right chunk
                            right_range = Some(Range::new(piece_range.begin(),
                                                          data.range.end() - piece_range.begin()));
                            debug!("split_to_width: case=splitting remainder with right range=%?",
                                   right_range);
                        }
                    }
                    pieces_processed_count += 1;
                    should_continue
                }

                let left_box = if left_range.length() > 0 {
                    Some(layout::text::adapt_textbox_with_range(self.d(), data.run, &left_range))
                } else { None };

                let right_box = right_range.map_default(None, |range: &Range| {
                    Some(layout::text::adapt_textbox_with_range(self.d(), data.run, range))
                });
                
                return if pieces_processed_count == 1 || left_box.is_none() {
                    SplitDidNotFit(left_box, right_box)
                } else {
                    SplitDidFit(left_box, right_box)
                }
            },
        }
    }

    /** In general, these functions are transitively impure because they
     * may cause glyphs to be allocated. For now, it's impure because of 
     * holder.get_image()
    */
    fn get_min_width(&mut self, _ctx: &LayoutContext) -> Au {
        match *self {
            // TODO: this should account for min/pref widths of the
            // box element in isolation. That includes
            // border/margin/padding but not child widths. The block
            // FlowContext will combine the width of this element and
            // that of its children to arrive at the context width.
            GenericBox(*) => Au(0),
            // TODO: consult CSS 'width', margin, border.
            // TODO: If image isn't available, consult 'width'.
            ImageBox(_, ref mut i) => Au::from_px(i.get_size().get_or_default(Size2D(0,0)).width),
            TextBox(_,d) => d.run.min_width_for_range(&d.range),
            UnscannedTextBox(*) => fail!(~"Shouldn't see unscanned boxes here.")
        }
    }

    fn get_pref_width(&mut self, _ctx: &LayoutContext) -> Au {
        match self {
            // TODO: this should account for min/pref widths of the
            // box element in isolation. That includes
            // border/margin/padding but not child widths. The block
            // FlowContext will combine the width of this element and
            // that of its children to arrive at the context width.
            &GenericBox(*) => Au(0),
            &ImageBox(_, ref mut i) => Au::from_px(i.get_size().get_or_default(Size2D(0,0)).width),

            // a text box cannot span lines, so assume that this is an unsplit text box.

            // TODO: If text boxes have been split to wrap lines, then
            // they could report a smaller pref width during incremental reflow.
            // maybe text boxes should report nothing, and the parent flow could
            // factor in min/pref widths of any text runs that it owns.
            &TextBox(_,d) => {
                let mut max_line_width: Au = Au(0);
                for d.run.iter_natural_lines_for_range(&d.range) |line_range| {
                    let mut line_width: Au = Au(0);
                    for d.run.glyphs.iter_glyphs_for_char_range(line_range) |_char_i, glyph| {
                        line_width += glyph.advance()
                    }
                    max_line_width = Au::max(max_line_width, line_width);
                }

                max_line_width
            },
            &UnscannedTextBox(*) => fail!(~"Shouldn't see unscanned boxes here.")
        }
    }

    /* Returns the amount of left, right "fringe" used by this
    box. This should be based on margin, border, padding, width. */
    fn get_used_width(&self) -> (Au, Au) {
        // TODO: this should actually do some computation!
        // See CSS 2.1, Section 10.3, 10.4.

        (Au(0), Au(0))
    }
    
    /* Returns the amount of left, right "fringe" used by this
    box. This should be based on margin, border, padding, width. */
    fn get_used_height(&self) -> (Au, Au) {
        // TODO: this should actually do some computation!
        // See CSS 2.1, Section 10.5, 10.6.

        (Au(0), Au(0))
    }

    /* The box formed by the content edge, as defined in CSS 2.1 Section 8.1.
       Coordinates are relative to the owning flow. */
    fn content_box(&mut self) -> Rect<Au> {
        let origin = {copy self.d().position.origin};
        match self {
            &ImageBox(_, ref mut i) => {
                let size = i.size();
                Rect {
                    origin: origin,
                    size:   Size2D(Au::from_px(size.width),
                                   Au::from_px(size.height))
                }
            },
            &GenericBox(*) => {
                copy self.d().position
                /* FIXME: The following hits an ICE for whatever reason

                let origin = self.d().position.origin;
                let size   = self.d().position.size;
                let (offset_left, offset_right) = self.get_used_width();
                let (offset_top, offset_bottom) = self.get_used_height();

                Rect {
                    origin: Point2D(origin.x + offset_left, origin.y + offset_top),
                    size:   Size2D(size.width - (offset_left + offset_right),
                                   size.height - (offset_top + offset_bottom))
                }*/
            },
            &TextBox(*) => {
                copy self.d().position
            },
            &UnscannedTextBox(*) => fail!(~"Shouldn't see unscanned boxes here.")
        }
    }

    /* The box formed by the border edge, as defined in CSS 2.1 Section 8.1.
       Coordinates are relative to the owning flow. */
    fn border_box(&mut self) -> Rect<Au> {
        // TODO: actually compute content_box + padding + border
        self.content_box()
    }

    /* The box fromed by the margin edge, as defined in CSS 2.1 Section 8.1.
       Coordinates are relative to the owning flow. */
    fn margin_box(&mut self) -> Rect<Au> {
        // TODO: actually compute content_box + padding + border + margin
        self.content_box()
    }

    fn style(&'self mut self) -> CompleteStyle<'self> {
        let d: &'self mut RenderBoxData = self.d();
        d.node.style()
    }

    fn with_style_of_nearest_element<R>(@mut self, f: &fn(CompleteStyle) -> R) -> R {
        let mut node = self.d().node;
        while !node.is_element() {
            node = node.parent_node().get();
        }
        f(node.style())
    }

    // TODO: to implement stacking contexts correctly, we need to
    // create a set of display lists, one per each layer of a stacking
    // context. (CSS 2.1, Section 9.9.1). Each box is passed the list
    // set representing the box's stacking context. When asked to
    // construct its constituent display items, each box puts its
    // DisplayItems into the correct stack layer (according to CSS 2.1
    // Appendix E).  and then builder flattens the list at the end.

    /* Methods for building a display list. This is a good candidate
       for a function pointer as the number of boxes explodes.

    # Arguments

    * `builder` - the display list builder which manages the coordinate system and options.
    * `dirty` - Dirty rectangle, in the coordinate system of the owning flow (self.ctx)
    * `origin` - Total offset from display list root flow to this box's owning flow
    * `list` - List to which items should be appended
    */
    fn build_display_list(@mut self, _builder: &DisplayListBuilder, dirty: &Rect<Au>,
                          offset: &Point2D<Au>, list: &Cell<DisplayList>) {

        let box_bounds = self.d().position;

        let abs_box_bounds = box_bounds.translate(offset);
        debug!("RenderBox::build_display_list at rel=%?, abs=%?: %s", 
               box_bounds, abs_box_bounds, self.debug_str());
        debug!("RenderBox::build_display_list: dirty=%?, offset=%?", dirty, offset);
        if abs_box_bounds.intersects(dirty) {
            debug!("RenderBox::build_display_list: intersected. Adding display item...");
        } else {
            debug!("RenderBox::build_display_list: Did not intersect...");
            return;
        }

        self.add_bgcolor_to_list(list, &abs_box_bounds); 

        let m = &mut *self;
        match m {
            &UnscannedTextBox(*) => fail!(~"Shouldn't see unscanned boxes here."),
            &TextBox(_,data) => {
                let nearest_ancestor_element = self.nearest_ancestor_element();
                let color = nearest_ancestor_element.style().color().to_gfx_color();
                let mut l = list.take(); // FIXME: this should use with_mut_ref when that appears
                l.append_item(~DisplayItem::new_Text(&abs_box_bounds,
                                                     ~data.run.serialize(),
                                                     data.range,
                                                     color));
                list.put_back(l);

                // debug frames for text box bounds
                debug!("%?", { 
                    // text box bounds
                    let mut l = list.take(); // FIXME: use with_mut_ref when that appears
                    l.append_item(~DisplayItem::new_Border(&abs_box_bounds,
                                                           Au::from_px(1),
                                                           rgb(0, 0, 200).to_gfx_color()));
                    // baseline "rect"
                    // TODO(Issue #221): create and use a Line display item for baseline.
                    let ascent = data.run.metrics_for_range(&data.range).ascent;
                    let baseline = Rect(abs_box_bounds.origin + Point2D(Au(0),ascent),
                                        Size2D(abs_box_bounds.size.width, Au(0)));

                    l.append_item(~DisplayItem::new_Border(&baseline,
                                                           Au::from_px(1),
                                                           rgb(0, 200, 0).to_gfx_color()));
                    list.put_back(l);
                    ; ()});
            },
            // TODO: items for background, border, outline
            &GenericBox(_) => {}
            &ImageBox(_, ref mut i) => {
                //let i: &mut ImageHolder = unsafe { cast::transmute(i) }; // Rust #5074
                match i.get_image() {
                    Some(image) => {
                        debug!("(building display list) building image box");
                        let mut l = list.take(); // FIXME: use with_mut_ref when available
                        l.append_item(~DisplayItem::new_Image(&abs_box_bounds,
                                                              arc::clone(&image)));
                        list.put_back(l);
                    }
                    None => {
                        /* No image data at all? Okay, add some fallback content instead. */
                        debug!("(building display list) no image :(");
                    }
                }
            }
        }

        self.add_border_to_list(list, &abs_box_bounds);
    }

    fn add_bgcolor_to_list(&mut self, list: &Cell<DisplayList>, abs_bounds: &Rect<Au>) {
        use std::cmp::FuzzyEq;

        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a RenderBox".
        let nearest_ancestor_element = self.nearest_ancestor_element();

        let bgcolor = nearest_ancestor_element.style().background_color();
        if !bgcolor.alpha.fuzzy_eq(&0.0) {
            let mut l = list.take(); // FIXME: use with_mut_ref when available
            l.append_item(~DisplayItem::new_SolidColor(abs_bounds, bgcolor.to_gfx_color()));
            list.put_back(l);
        }
    }

    fn add_border_to_list(&mut self, list: &Cell<DisplayList>, abs_bounds: &Rect<Au>) {
        if !self.d().node.is_element() { return }

        let top_width = self.style().border_top_width();
        let right_width = self.style().border_right_width();
        let bottom_width = self.style().border_bottom_width();
        let left_width = self.style().border_left_width();

        match (top_width, right_width, bottom_width, left_width) {
            (CSSBorderWidthLength(Px(top)),
             CSSBorderWidthLength(Px(right)),
             CSSBorderWidthLength(Px(bottom)),
             CSSBorderWidthLength(Px(left))) => {
                let top_au = Au::from_frac_px(top);
                let right_au = Au::from_frac_px(right);
                let bottom_au = Au::from_frac_px(bottom);
                let left_au = Au::from_frac_px(left);

                let all_widths_equal = [top_au, right_au, bottom_au].all(|a| *a == left_au);

                if all_widths_equal {
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
                    let mut l = list.take(); // FIXME: use with_mut_ref when available
                    l.append_item(~DisplayItem::new_Border(&bounds, border_width, color));
                    list.put_back(l);
                } else {
                    warn!("ignoring unimplemented border widths");
                }
            }
            (CSSBorderWidthMedium,
             CSSBorderWidthMedium,
             CSSBorderWidthMedium,
             CSSBorderWidthMedium) => {
                // FIXME: This seems to be the default for non-root nodes. For now we'll ignore it
                warn!("ignoring medium border widths");
            }
            _ => warn!("ignoring unimplemented border widths")
        }
    }

    // Converts this node's ComputedStyle to a font style used in the graphics code.
    fn font_style(@mut self) -> FontStyle {
        do self.with_style_of_nearest_element |my_style| {
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
                CSSFontSizeLength(Px(l)) |
                CSSFontSizeLength(Pt(l)) => l,
                CSSFontSizeLength(Em(l)) => l,
                _ => 16f
            };
            debug!("(font style) font size: `%f`", font_size);

            let italic, oblique;
            match my_style.font_style() {
                CSSFontStyleNormal  => { italic = false; oblique = false; }
                CSSFontStyleItalic  => { italic = true;  oblique = false; }
                CSSFontStyleOblique => { italic = false; oblique = true;  }
            }

            FontStyle {
                pt_size: font_size,
                weight: FontWeight300,
                italic: italic,
                oblique: oblique,
                families: font_families,
            }
        }
    }

    // Converts this node's ComputedStyle to a text alignment used in the inline layout code.
    fn text_align(@mut self) -> CSSTextAlign {
        do self.with_style_of_nearest_element |my_style| {
            my_style.text_align()
        }
    }
}

impl BoxedMutDebugMethods for RenderBox {
    fn dump(@mut self) {
        self.dump_indent(0u);
    }

    /* Dumps the node tree, for debugging, with indentation. */
    fn dump_indent(@mut self, indent: uint) {
        let mut s = ~"";
        for uint::range(0u, indent) |_i| {
            s += ~"    ";
        }

        s += self.debug_str();
        debug!("%s", s);
    }

    fn debug_str(@mut self) -> ~str {
        let borrowed_self : &mut RenderBox = self; // FIXME: borrow checker workaround
        let repr = match borrowed_self {
            &GenericBox(*) => ~"GenericBox",
            &ImageBox(*) => ~"ImageBox",
            &TextBox(_,d) => fmt!("TextBox(text=%s)", str::substr(d.run.text, d.range.begin(), d.range.length())),
            &UnscannedTextBox(_, ref s) => {
                let s = s; // FIXME: borrow checker workaround
                fmt!("UnscannedTextBox(%s)", *s)
            }
        };

        let borrowed_self : &mut RenderBox = self; // FIXME: borrow checker workaround
        let id = borrowed_self.d().id;
        fmt!("box b%?: %?", id, repr)
    }
}

// Other methods
impl RenderBox {
    /// Returns the nearest ancestor-or-self element node. Infallible.
    fn nearest_ancestor_element(&mut self) -> AbstractNode {
        let mut node = self.d().node;
        while !node.is_element() {
            match node.parent_node() {
                None => fail!(~"no nearest element?!"),
                Some(parent) => node = parent,
            }
        }
        node
    }
}

// FIXME: This belongs somewhere else
trait ToGfxColor {
    fn to_gfx_color(&self) -> gfx::color::Color;
}

impl ToGfxColor for Color {
    fn to_gfx_color(&self) -> gfx::color::Color {
        gfx::color::rgba(self.red,
                         self.green,
                         self.blue,
                         self.alpha)
    }
}
