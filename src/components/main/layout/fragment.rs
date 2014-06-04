/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Box` type, which represents the leaves of the layout tree.

#![deny(unsafe_block)]

use css::node_style::StyledNode;
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::floats::{ClearBoth, ClearLeft, ClearRight, ClearType};
use layout::flow::Flow;
use layout::flow;
use layout::inline::{InlineFragmentContext, InlineMetrics};
use layout::model::{Auto, IntrinsicWidths, MaybeAuto, Specified, specified};
use layout::model;
use layout::text;
use layout::util::{OpaqueNodeMethods, ToGfxColor};
use layout::wrapper::{TLayoutNode, ThreadSafeLayoutNode};

use geom::{Point2D, Rect, Size2D, SideOffsets2D};
use geom::approxeq::ApproxEq;
use gfx::color::rgb;
use gfx::display_list::{BackgroundAndBorderLevel, BaseDisplayItem, BorderDisplayItem};
use gfx::display_list::{BorderDisplayItemClass, ClipDisplayItem, ClipDisplayItemClass};
use gfx::display_list::{ContentStackingLevel, DisplayItem, DisplayList, ImageDisplayItem};
use gfx::display_list::{ImageDisplayItemClass, LineDisplayItem};
use gfx::display_list::{LineDisplayItemClass, OpaqueNode, PseudoDisplayItemClass};
use gfx::display_list::{SolidColorDisplayItem, SolidColorDisplayItemClass, StackingLevel};
use gfx::display_list::{TextDecorations, TextDisplayItem, TextDisplayItemClass};
use gfx::font::FontStyle;
use gfx::text::glyph::CharIndex;
use gfx::text::text_run::TextRun;
use servo_msg::constellation_msg::{ConstellationChan, FrameRectMsg, PipelineId, SubpageId};
use servo_net::image::holder::{ImageHolder, LocalImageCacheHandle};
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::range::*;
use servo_util::namespace;
use servo_util::smallvec::SmallVec;
use servo_util::str::is_whitespace;
use std::fmt;
use std::from_str::FromStr;
use std::iter::AdditiveIterator;
use std::mem;
use std::num::Zero;
use style::{ComputedValues, TElement, TNode, cascade_anonymous};
use style::computed_values::{LengthOrPercentageOrAuto, overflow, LPA_Auto, background_attachment};
use style::computed_values::{background_repeat, border_style, clear, position, text_align};
use style::computed_values::{text_decoration, vertical_align, visibility, white_space};
use sync::Arc;
use url::Url;

/// Fragments (`struct Fragment`) are the leaves of the layout tree. They cannot position themselves. In
/// general, fragments do not have a simple correspondence with CSS fragments in the specification:
///
/// * Several fragments may correspond to the same CSS box or DOM node. For example, a CSS text box
/// broken across two lines is represented by two fragments.
///
/// * Some CSS fragments are not created at all, such as some anonymous block fragments induced by inline
///   fragments with block-level sibling fragments. In that case, Servo uses an `InlineFlow` with
///   `BlockFlow` siblings; the `InlineFlow` is block-level, but not a block container. It is
///   positioned as if it were a block fragment, but its children are positioned according to inline
///   flow.
///
/// A `GenericFragment` is an empty fragment that contributes only borders, margins, padding, and
/// backgrounds. It is analogous to a CSS nonreplaced content box.
///
/// A fragment's type influences how its styles are interpreted during layout. For example, replaced
/// content such as images are resized differently from tables, text, or other content. Different
/// types of fragments may also contain custom data; for example, text fragments contain text.
///
/// FIXME(#2260, pcwalton): This can be slimmed down some.
#[deriving(Clone)]
pub struct Fragment {
    /// An opaque reference to the DOM node that this `Fragment` originates from.
    pub node: OpaqueNode,

    /// The CSS style of this fragment.
    pub style: Arc<ComputedValues>,

    /// The position of this fragment relative to its owning flow.
    /// The size includes padding and border, but not margin.
    pub border_box: Rect<Au>,

    /// The sum of border and padding; i.e. the distance from the edge of the border box to the
    /// content edge of the fragment.
    pub border_padding: SideOffsets2D<Au>,

    /// The margin of the content box.
    pub margin: SideOffsets2D<Au>,

    /// Info specific to the kind of fragment. Keep this enum small.
    pub specific: SpecificFragmentInfo,

    /// New-line chracter(\n)'s positions(relative, not absolute)
    ///
    /// FIXME(#2260, pcwalton): This is very inefficient; remove.
    pub new_line_pos: Vec<CharIndex>,
}

/// Info specific to the kind of fragment. Keep this enum small.
#[deriving(Clone)]
pub enum SpecificFragmentInfo {
    GenericFragment,
    ImageFragment(ImageFragmentInfo),
    IframeFragment(IframeFragmentInfo),
    ScannedTextFragment(ScannedTextFragmentInfo),
    TableFragment,
    TableCellFragment,
    TableColumnFragment(TableColumnFragmentInfo),
    TableRowFragment,
    TableWrapperFragment,
    UnscannedTextFragment(UnscannedTextFragmentInfo),
}

/// A fragment that represents a replaced content image and its accompanying borders, shadows, etc.
#[deriving(Clone)]
pub struct ImageFragmentInfo {
    /// The image held within this fragment.
    pub image: ImageHolder,
    pub computed_width: Option<Au>,
    pub computed_height: Option<Au>,
    pub dom_width: Option<Au>,
    pub dom_height: Option<Au>,
}

impl ImageFragmentInfo {
    /// Creates a new image fragment from the given URL and local image cache.
    ///
    /// FIXME(pcwalton): The fact that image fragments store the cache in the fragment makes little sense to
    /// me.
    pub fn new(node: &ThreadSafeLayoutNode,
               image_url: Url,
               local_image_cache: LocalImageCacheHandle)
               -> ImageFragmentInfo {
        fn convert_length(node: &ThreadSafeLayoutNode, name: &str) -> Option<Au> {
            let element = node.as_element();
            element.get_attr(&namespace::Null, name).and_then(|string| {
                let n: Option<int> = FromStr::from_str(string);
                n
            }).and_then(|pixels| Some(Au::from_px(pixels)))
        }

        ImageFragmentInfo {
            image: ImageHolder::new(image_url, local_image_cache),
            computed_width: None,
            computed_height: None,
            dom_width: convert_length(node,"width"),
            dom_height: convert_length(node,"height"),
        }
    }

    /// Returns the calculated width of the image, accounting for the width attribute.
    pub fn computed_width(&self) -> Au {
        self.computed_width.expect("image width is not computed yet!")
    }

    /// Returns the original width of the image.
    pub fn image_width(&mut self) -> Au {
        let image_ref = &mut self.image;
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
        match self.computed_height {
            Some(height) => height,
            None => fail!("image height is not computed yet!"),
        }
    }

    /// Returns the original height of the image.
    pub fn image_height(&mut self) -> Au {
        let image_ref = &mut self.image;
        Au::from_px(image_ref.get_size().unwrap_or(Size2D(0,0)).height)
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
}

impl ScannedTextFragmentInfo {
    /// Creates the information specific to a scanned text fragment from a range and a text run.
    pub fn new(run: Arc<Box<TextRun>>, range: Range<CharIndex>) -> ScannedTextFragmentInfo {
        ScannedTextFragmentInfo {
            run: run,
            range: range,
        }
    }
}

#[deriving(Show)]
pub struct SplitInfo {
    // TODO(bjz): this should only need to be a single character index, but both values are
    // currently needed for splitting in the `inline::try_append_*` functions.
    pub range: Range<CharIndex>,
    pub width: Au,
}

impl SplitInfo {
    fn new(range: Range<CharIndex>, info: &ScannedTextFragmentInfo) -> SplitInfo {
        SplitInfo {
            range: range,
            width: info.run.advance_for_range(&range),
        }
    }
}

/// Data for an unscanned text fragment. Unscanned text fragments are the results of flow construction that
/// have not yet had their width determined.
#[deriving(Clone)]
pub struct UnscannedTextFragmentInfo {
    /// The text inside the fragment.
    pub text: ~str,
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
    pub fn from_text(text: ~str) -> UnscannedTextFragmentInfo {
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
            element.get_attr(&namespace::Null, "span").and_then(|string| {
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
    /// Arguments:
    ///
    ///   * `constructor`: The flow constructor.
    ///
    ///   * `node`: The node to create a fragment for.
    pub fn new(constructor: &mut FlowConstructor, node: &ThreadSafeLayoutNode) -> Fragment {
        Fragment {
            node: OpaqueNodeMethods::from_thread_safe_layout_node(node),
            style: node.style().clone(),
            border_box: Rect::zero(),
            border_padding: Zero::zero(),
            margin: Zero::zero(),
            specific: constructor.build_specific_fragment_info_for_node(node),
            new_line_pos: vec!(),
        }
    }

    /// Constructs a new `Fragment` instance from a specific info.
    pub fn new_from_specific_info(node: &ThreadSafeLayoutNode, specific: SpecificFragmentInfo) -> Fragment {
        Fragment {
            node: OpaqueNodeMethods::from_thread_safe_layout_node(node),
            style: node.style().clone(),
            border_box: Rect::zero(),
            border_padding: Zero::zero(),
            margin: Zero::zero(),
            specific: specific,
            new_line_pos: vec!(),
        }
    }

    /// Constructs a new `Fragment` instance for an anonymous table object.
    pub fn new_anonymous_table_fragment(node: &ThreadSafeLayoutNode, specific: SpecificFragmentInfo) -> Fragment {
        // CSS 2.1 § 17.2.1 This is for non-inherited properties on anonymous table fragments
        // example:
        //
        //     <div style="display: table">
        //         Foo
        //     </div>
        //
        // Anonymous table fragments, TableRowFragment and TableCellFragment, are generated around `Foo`, but it shouldn't inherit the border.

        let node_style = cascade_anonymous(&**node.style());
        Fragment {
            node: OpaqueNodeMethods::from_thread_safe_layout_node(node),
            style: Arc::new(node_style),
            border_box: Rect::zero(),
            border_padding: Zero::zero(),
            margin: Zero::zero(),
            specific: specific,
            new_line_pos: vec!(),
        }
    }

    /// Constructs a new `Fragment` instance from an opaque node.
    pub fn from_opaque_node_and_style(node: OpaqueNode,
                                      style: Arc<ComputedValues>,
                                      specific: SpecificFragmentInfo)
                                      -> Fragment {
        Fragment {
            node: node,
            style: style,
            border_box: Rect::zero(),
            border_padding: Zero::zero(),
            margin: Zero::zero(),
            specific: specific,
            new_line_pos: vec!(),
        }
    }

    /// Returns a debug ID of this fragment. This ID should not be considered stable across multiple
    /// layouts or fragment manipulations.
    pub fn debug_id(&self) -> uint {
        self as *Fragment as uint
    }

    /// Transforms this fragment into another fragment of the given type, with the given size, preserving all
    /// the other data.
    pub fn transform(&self, size: Size2D<Au>, specific: SpecificFragmentInfo) -> Fragment {
        Fragment {
            node: self.node,
            style: self.style.clone(),
            border_box: Rect(self.border_box.origin, size),
            border_padding: self.border_padding,
            margin: self.margin,
            specific: specific,
            new_line_pos: self.new_line_pos.clone(),
        }
    }

    /// Uses the style only to estimate the intrinsic widths. These may be modified for text or
    /// replaced elements.
    fn style_specified_intrinsic_width(&self) -> IntrinsicWidths {
        let (use_margins, use_padding) = match self.specific {
            GenericFragment | IframeFragment(_) | ImageFragment(_) => (true, true),
            TableFragment | TableCellFragment => (false, true),
            TableWrapperFragment => (true, false),
            TableRowFragment => (false, false),
            ScannedTextFragment(_) | TableColumnFragment(_) | UnscannedTextFragment(_) => {
                // Styles are irrelevant for these kinds of fragments.
                return IntrinsicWidths::new()
            }
        };

        let style = self.style();
        let width = MaybeAuto::from_style(style.get_box().width, Au::new(0)).specified_or_zero();

        let (margin_left, margin_right) = if use_margins {
            (MaybeAuto::from_style(style.get_margin().margin_left, Au(0)).specified_or_zero(),
             MaybeAuto::from_style(style.get_margin().margin_right, Au(0)).specified_or_zero())
        } else {
            (Au(0), Au(0))
        };

        let (padding_left, padding_right) = if use_padding {
            (model::specified(style.get_padding().padding_left, Au(0)),
             model::specified(style.get_padding().padding_right, Au(0)))
        } else {
            (Au(0), Au(0))
        };

        // FIXME(#2261, pcwalton): This won't work well for inlines: is this OK?
        let border = self.border_width(None);
        let surround_width = margin_left + margin_right + padding_left + padding_right +
                border.horizontal();

        IntrinsicWidths {
            minimum_width: width,
            preferred_width: width,
            surround_width: surround_width,
        }
    }

    pub fn calculate_line_height(&self, font_size: Au) -> Au {
        text::line_height_from_style(self.style(), font_size)
    }

    /// Returns the sum of the widths of all the borders of this fragment. This is private because
    /// it should only be called during intrinsic width computation or computation of
    /// `border_padding`. Other consumers of this information should simply consult that field.
    #[inline]
    fn border_width(&self, inline_fragment_context: Option<InlineFragmentContext>)
                    -> SideOffsets2D<Au> {
        match inline_fragment_context {
            None => model::border_from_style(self.style()),
            Some(inline_fragment_context) => {
                inline_fragment_context.ranges().map(|range| range.border()).sum()
            }
        }
    }

    /// Computes the border, padding, and vertical margins from the containing block width and the
    /// style. After this call, the `border_padding` and the vertical direction of the `margin`
    /// field will be correct.
    pub fn compute_border_padding_margins(&mut self,
                                          containing_block_width: Au,
                                          inline_fragment_context: Option<InlineFragmentContext>) {
        // Compute vertical margins. Note that this value will be ignored by layout if the style
        // specifies `auto`.
        match self.specific {
            TableFragment | TableCellFragment | TableRowFragment | TableColumnFragment(_) => {
                self.margin.top = Au(0);
                self.margin.bottom = Au(0)
            }
            _ => {
                // NB: Percentages are relative to containing block width (not height) per CSS 2.1.
                self.margin.top =
                    MaybeAuto::from_style(self.style().get_margin().margin_top,
                                          containing_block_width).specified_or_zero();
                self.margin.bottom =
                    MaybeAuto::from_style(self.style().get_margin().margin_bottom,
                                          containing_block_width).specified_or_zero()
            }
        }

        // Compute border.
        let border = match inline_fragment_context {
            None => model::border_from_style(self.style()),
            Some(inline_fragment_context) => {
                inline_fragment_context.ranges().map(|range| range.border()).sum()
            }
        };

        // Compute padding.
        let padding = match self.specific {
            TableColumnFragment(_) | TableRowFragment | TableWrapperFragment => Zero::zero(),
            _ => {
                match inline_fragment_context {
                    None => model::padding_from_style(self.style(), containing_block_width),
                    Some(inline_fragment_context) => {
                        inline_fragment_context.ranges().map(|range| range.padding()).sum()
                    }
                }
            }
        };

        self.border_padding = border + padding
    }

    // Return offset from original position because of `position: relative`.
    pub fn relative_position(&self,
                             container_block_size: &Size2D<Au>,
                             inline_fragment_context: Option<InlineFragmentContext>)
                             -> Point2D<Au> {
        fn left_right(style: &ComputedValues, block_width: Au) -> Au {
            // TODO(ksh8281) : consider RTL(right-to-left) culture
            match (style.get_positionoffsets().left, style.get_positionoffsets().right) {
                (LPA_Auto, _) => {
                    -MaybeAuto::from_style(style.get_positionoffsets().right, block_width)
                        .specified_or_zero()
                }
                (_, _) => {
                    MaybeAuto::from_style(style.get_positionoffsets().left, block_width)
                        .specified_or_zero()
                }
            }
        }

        fn top_bottom(style: &ComputedValues,block_height: Au) -> Au {
            match (style.get_positionoffsets().top, style.get_positionoffsets().bottom) {
                (LPA_Auto, _) => {
                    -MaybeAuto::from_style(style.get_positionoffsets().bottom, block_height)
                        .specified_or_zero()
                }
                (_, _) => {
                    MaybeAuto::from_style(style.get_positionoffsets().top, block_height)
                        .specified_or_zero()
                }
            }
        }

        // Go over the ancestor fragments and add all relative offsets (if any).
        let mut rel_pos: Point2D<Au> = Zero::zero();
        match inline_fragment_context {
            None => {
                if self.style().get_box().position == position::relative {
                    rel_pos.x = rel_pos.x + left_right(self.style(), container_block_size.width);
                    rel_pos.y = rel_pos.y + top_bottom(self.style(), container_block_size.height);
                }
            }
            Some(inline_fragment_context) => {
                for range in inline_fragment_context.ranges() {
                    if range.style.get_box().position == position::relative {
                        rel_pos.x = rel_pos.x + left_right(&*range.style,
                                                           container_block_size.width);
                        rel_pos.y = rel_pos.y + top_bottom(&*range.style,
                                                           container_block_size.height);
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

    /// Converts this fragment's computed style to a font style used for rendering.
    pub fn font_style(&self) -> FontStyle {
        text::computed_style_to_font_style(self.style())
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

    /// Returns the left offset from margin edge to content edge.
    ///
    /// FIXME(#2262, pcwalton): I think this method is pretty bogus, because it won't work for
    /// inlines.
    pub fn left_offset(&self) -> Au {
        match self.specific {
            TableWrapperFragment => self.margin.left,
            TableFragment | TableCellFragment | TableRowFragment => self.border_padding.left,
            TableColumnFragment(_) => Au(0),
            _ => self.margin.left + self.border_padding.left,
        }
    }

    /// Returns true if this element can be split. This is true for text fragments.
    pub fn can_split(&self) -> bool {
        match self.specific {
            ScannedTextFragment(..) => true,
            _ => false,
        }
    }

    /// Adds the display items necessary to paint the background of this fragment to the display
    /// list if necessary.
    pub fn build_display_list_for_background_if_applicable(&self,
                                                           list: &mut DisplayList,
                                                           layout_context: &LayoutContext,
                                                           level: StackingLevel,
                                                           absolute_bounds: &Rect<Au>) {
        // FIXME: This causes a lot of background colors to be displayed when they are clearly not
        // needed. We could use display list optimization to clean this up, but it still seems
        // inefficient. What we really want is something like "nearest ancestor element that
        // doesn't have a fragment".
        let style = self.style();
        let background_color = style.resolve_color(style.get_background().background_color);
        if !background_color.alpha.approx_eq(&0.0) {
            let display_item = box SolidColorDisplayItem {
                base: BaseDisplayItem::new(*absolute_bounds, self.node, level),
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

        let mut holder = ImageHolder::new(image_url.clone(), layout_context.image_cache.clone());
        let image = match holder.get_image() {
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

        // Adjust bounds for `background-position` and `background-attachment`.
        let mut bounds = *absolute_bounds;
        let horizontal_position = model::specified(background.background_position.horizontal,
                                                   bounds.size.width);
        let vertical_position = model::specified(background.background_position.vertical,
                                                 bounds.size.height);

        let clip_display_item;
        match background.background_attachment {
            background_attachment::scroll => {
                clip_display_item = None;
                bounds.origin.x = bounds.origin.x + horizontal_position;
                bounds.origin.y = bounds.origin.y + vertical_position;
                bounds.size.width = bounds.size.width - horizontal_position;
                bounds.size.height = bounds.size.height - vertical_position;
            }
            background_attachment::fixed => {
                clip_display_item = Some(box ClipDisplayItem {
                    base: BaseDisplayItem::new(bounds, self.node, level),
                    children: DisplayList::new(),
                });

                bounds = Rect {
                    origin: Point2D(horizontal_position, vertical_position),
                    size: Size2D(bounds.origin.x + bounds.size.width,
                                 bounds.origin.y + bounds.size.height),
                }
            }
        }

        // Adjust sizes for `background-repeat`.
        match background.background_repeat {
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
        let image_display_item = ImageDisplayItemClass(box ImageDisplayItem {
            base: BaseDisplayItem::new(bounds, self.node, level),
            image: image.clone(),
            stretch_size: Size2D(Au::from_px(image.width as int),
                                 Au::from_px(image.height as int)),
        });

        match clip_display_item {
            None => list.push(image_display_item),
            Some(mut clip_display_item) => {
                clip_display_item.children.push(image_display_item);
                list.push(ClipDisplayItemClass(clip_display_item))
            }
        }
    }

    /// Adds the display items necessary to paint the borders of this fragment to a display list if
    /// necessary.
    pub fn build_display_list_for_borders_if_applicable(&self,
                                                        list: &mut DisplayList,
                                                        abs_bounds: &Rect<Au>,
                                                        level: StackingLevel,
                                                        inline_fragment_context:
                                                            Option<InlineFragmentContext>) {
        // Fast path.
        let border = self.border_width(inline_fragment_context);
        if border == Zero::zero() {
            return
        }

        let style = self.style();
        let top_color = style.resolve_color(style.get_border().border_top_color);
        let right_color = style.resolve_color(style.get_border().border_right_color);
        let bottom_color = style.resolve_color(style.get_border().border_bottom_color);
        let left_color = style.resolve_color(style.get_border().border_left_color);

        // Append the border to the display list.
        let border_display_item = box BorderDisplayItem {
            base: BaseDisplayItem::new(*abs_bounds, self.node, level),
            border: border,
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
                                             text_fragment: &ScannedTextFragmentInfo) {
        let fragment_bounds = self.border_box;
        let absolute_fragment_bounds = fragment_bounds.translate(&flow_origin);

        // Compute the text fragment bounds and draw a border surrounding them.
        let debug_border = SideOffsets2D::new_all_same(Au::from_px(1));

        let border_display_item = box BorderDisplayItem {
            base: BaseDisplayItem::new(absolute_fragment_bounds, self.node, ContentStackingLevel),
            border: debug_border,
            color: SideOffsets2D::new_all_same(rgb(0, 0, 200)),
            style: SideOffsets2D::new_all_same(border_style::solid)
        };
        display_list.push(BorderDisplayItemClass(border_display_item));

        // Draw a rectangle representing the baselines.
        let ascent = text_fragment.run.ascent();
        let baseline = Rect(absolute_fragment_bounds.origin + Point2D(Au(0), ascent),
                            Size2D(absolute_fragment_bounds.size.width, Au(0)));

        let line_display_item = box LineDisplayItem {
            base: BaseDisplayItem::new(baseline, self.node, ContentStackingLevel),
            color: rgb(0, 200, 0),
            style: border_style::dashed,
        };
        display_list.push(LineDisplayItemClass(line_display_item));
    }

    fn build_debug_borders_around_fragment(&self,
                                      display_list: &mut DisplayList,
                                      flow_origin: Point2D<Au>) {
        let fragment_bounds = self.border_box;
        let absolute_fragment_bounds = fragment_bounds.translate(&flow_origin);

        // This prints a debug border around the border of this fragment.
        let debug_border = SideOffsets2D::new_all_same(Au::from_px(1));

        let border_display_item = box BorderDisplayItem {
            base: BaseDisplayItem::new(absolute_fragment_bounds, self.node, ContentStackingLevel),
            border: debug_border,
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
    pub fn build_display_list(&self,
                              display_list: &mut DisplayList,
                              layout_context: &LayoutContext,
                              flow_origin: Point2D<Au>,
                              background_and_border_level: BackgroundAndBorderLevel,
                              inline_fragment_context: Option<InlineFragmentContext>)
                              -> ChildDisplayListAccumulator {
        // Fragment position wrt to the owning flow.
        let fragment_bounds = self.border_box;
        let absolute_fragment_bounds = fragment_bounds.translate(&flow_origin);
        debug!("Fragment::build_display_list at rel={}, abs={}: {}",
               fragment_bounds,
               absolute_fragment_bounds,
               self);
        debug!("Fragment::build_display_list: dirty={}, flow_origin={}",
               layout_context.dirty,
               flow_origin);

        let mut accumulator = ChildDisplayListAccumulator::new(self.style(),
                                                               absolute_fragment_bounds,
                                                               self.node,
                                                               ContentStackingLevel);
        if self.style().get_inheritedbox().visibility != visibility::visible {
            return accumulator
        }

        if !absolute_fragment_bounds.intersects(&layout_context.dirty) {
            debug!("Fragment::build_display_list: Did not intersect...");
            return accumulator
        }

        debug!("Fragment::build_display_list: intersected. Adding display item...");

        {
            let level =
                StackingLevel::from_background_and_border_level(background_and_border_level);

            // Add a pseudo-display item for content box queries. This is a very bogus thing to do.
            let base_display_item = box BaseDisplayItem::new(absolute_fragment_bounds, self.node, level);
            display_list.push(PseudoDisplayItemClass(base_display_item));

            // Add the background to the list, if applicable.
            self.build_display_list_for_background_if_applicable(display_list,
                                                                 layout_context,
                                                                 level,
                                                                 &absolute_fragment_bounds);

            // Add a border, if applicable.
            //
            // TODO: Outlines.
            self.build_display_list_for_borders_if_applicable(display_list,
                                                              &absolute_fragment_bounds,
                                                              level,
                                                              inline_fragment_context);
        }

        // Add a clip, if applicable.
        match self.specific {
            UnscannedTextFragment(_) => fail!("Shouldn't see unscanned fragments here."),
            TableColumnFragment(_) => fail!("Shouldn't see table column fragments here."),
            ScannedTextFragment(ref text_fragment) => {
                // Compute text color.
                let text_color = self.style().get_color().color.to_gfx_color();

                // Compute text decorations.
                let text_decorations_in_effect = self.style()
                                                     .get_inheritedtext()
                                                     ._servo_text_decorations_in_effect;
                let text_decorations = TextDecorations {
                    underline: text_decorations_in_effect.underline.map(|c| c.to_gfx_color()),
                    overline: text_decorations_in_effect.overline.map(|c| c.to_gfx_color()),
                    line_through: text_decorations_in_effect.line_through
                                                            .map(|c| c.to_gfx_color()),
                };

                let mut bounds = absolute_fragment_bounds.clone();
                bounds.origin.x = bounds.origin.x + self.border_padding.left;
                bounds.size.width = bounds.size.width - self.border_padding.horizontal();

                // Create the text fragment.
                let text_display_item = box TextDisplayItem {
                    base: BaseDisplayItem::new(bounds, self.node, ContentStackingLevel),
                    text_run: text_fragment.run.clone(),
                    range: text_fragment.range,
                    text_color: text_color,
                    text_decorations: text_decorations,
                };
                accumulator.push(display_list, TextDisplayItemClass(text_display_item));

                // Draw debug frames for text bounds.
                //
                // FIXME(#2263, pcwalton): This is a bit of an abuse of the logging infrastructure.
                // We should have a real `SERVO_DEBUG` system.
                debug!("{:?}", self.build_debug_borders_around_text_fragments(display_list,
                                                                          flow_origin,
                                                                          text_fragment))
            },
            GenericFragment | IframeFragment(..) | TableFragment | TableCellFragment | TableRowFragment |
            TableWrapperFragment => {
                // FIXME(pcwalton): This is a bit of an abuse of the logging infrastructure. We
                // should have a real `SERVO_DEBUG` system.
                debug!("{:?}", self.build_debug_borders_around_fragment(display_list, flow_origin))
            },
            ImageFragment(_) => {
                let mut bounds = absolute_fragment_bounds.clone();
                bounds.origin.x = bounds.origin.x + self.border_padding.left;
                bounds.origin.y = bounds.origin.y + self.border_padding.top;
                bounds.size.width = bounds.size.width - self.border_padding.horizontal();
                bounds.size.height = bounds.size.height - self.border_padding.vertical();

                match self.specific {
                    ImageFragment(ref image_fragment) => {
                        let image_ref = &image_fragment.image;
                        match image_ref.get_image_if_present() {
                            Some(image) => {
                                debug!("(building display list) building image fragment");

                                // Place the image into the display list.
                                let image_display_item = box ImageDisplayItem {
                                    base: BaseDisplayItem::new(bounds,
                                                               self.node,
                                                               ContentStackingLevel),
                                    image: image.clone(),
                                    stretch_size: bounds.size,
                                };
                                accumulator.push(display_list,
                                                 ImageDisplayItemClass(image_display_item))
                            }
                            None => {
                                // No image data at all? Do nothing.
                                //
                                // TODO: Add some kind of placeholder image.
                                debug!("(building display list) no image :(");
                            }
                        }
                    }
                    _ => fail!("shouldn't get here"),
                }

                // FIXME(pcwalton): This is a bit of an abuse of the logging
                // infrastructure. We should have a real `SERVO_DEBUG` system.
                debug!("{:?}", self.build_debug_borders_around_fragment(display_list, flow_origin))
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
            IframeFragment(ref iframe_fragment) => {
                self.finalize_position_and_size_of_iframe(iframe_fragment, flow_origin, layout_context)
            }
            _ => {}
        }

        accumulator
    }

    /// Returns the intrinsic widths of this fragment.
    pub fn intrinsic_widths(&mut self, inline_fragment_context: Option<InlineFragmentContext>)
                            -> IntrinsicWidths {
        let mut result = self.style_specified_intrinsic_width();

        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment | TableColumnFragment(_) | TableRowFragment |
            TableWrapperFragment => {}
            ImageFragment(ref mut image_fragment_info) => {
                let image_width = image_fragment_info.image_width();
                result.minimum_width = geometry::max(result.minimum_width, image_width);
                result.preferred_width = geometry::max(result.preferred_width, image_width);
            }
            ScannedTextFragment(ref text_fragment_info) => {
                let range = &text_fragment_info.range;
                let min_line_width = text_fragment_info.run.min_width_for_range(range);

                let mut max_line_width = Au::new(0);
                for line_range in text_fragment_info.run.iter_natural_lines_for_range(range) {
                    let line_metrics = text_fragment_info.run.metrics_for_range(&line_range);
                    max_line_width = Au::max(max_line_width, line_metrics.advance_width);
                }

                result.minimum_width = geometry::max(result.minimum_width, min_line_width);
                result.preferred_width = geometry::max(result.preferred_width, max_line_width);
            }
            UnscannedTextFragment(..) => fail!("Unscanned text fragments should have been scanned by now!"),
        }

        // Take borders and padding for parent inline fragments into account, if necessary.
        match inline_fragment_context {
            None => {}
            Some(context) => {
                for range in context.ranges() {
                    let border_width = range.border().horizontal();
                    let padding_width = range.padding().horizontal();
                    result.minimum_width = result.minimum_width + border_width + padding_width;
                    result.preferred_width = result.preferred_width + border_width + padding_width;
                }
            }
        }

        result
    }


    /// TODO: What exactly does this function return? Why is it Au(0) for GenericFragment?
    pub fn content_width(&self) -> Au {
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment | TableRowFragment |
            TableWrapperFragment => Au(0),
            ImageFragment(ref image_fragment_info) => {
                image_fragment_info.computed_width()
            }
            ScannedTextFragment(ref text_fragment_info) => {
                let (range, run) = (&text_fragment_info.range, &text_fragment_info.run);
                let text_bounds = run.metrics_for_range(range).bounding_box;
                text_bounds.size.width
            }
            TableColumnFragment(_) => fail!("Table column fragments do not have width"),
            UnscannedTextFragment(_) => fail!("Unscanned text fragments should have been scanned by now!"),
        }
    }

    /// Returns, and computes, the height of this fragment.
    pub fn content_height(&self) -> Au {
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment | TableRowFragment |
            TableWrapperFragment => Au(0),
            ImageFragment(ref image_fragment_info) => {
                image_fragment_info.computed_height()
            }
            ScannedTextFragment(ref text_fragment_info) => {
                // Compute the height based on the line-height and font size.
                //
                // FIXME(pcwalton): Shouldn't we use the value of the `font-size` property below
                // instead of the bounding box of the text run?
                let (range, run) = (&text_fragment_info.range, &text_fragment_info.run);
                let text_bounds = run.metrics_for_range(range).bounding_box;
                let em_size = text_bounds.size.height;
                self.calculate_line_height(em_size)
            }
            TableColumnFragment(_) => fail!("Table column fragments do not have height"),
            UnscannedTextFragment(_) => fail!("Unscanned text fragments should have been scanned by now!"),
        }
    }

    /// Returns the dimensions of the content box.
    ///
    /// This is marked `#[inline]` because it is frequently called when only one or two of the
    /// values are needed and that will save computation.
    #[inline]
    pub fn content_box(&self) -> Rect<Au> {
        Rect {
            origin: Point2D(self.border_box.origin.x + self.border_padding.left,
                            self.border_box.origin.y + self.border_padding.top),
            size: Size2D(self.border_box.size.width - self.border_padding.horizontal(),
                         self.border_box.size.height - self.border_padding.vertical()),
        }
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
            TableColumnFragment(_) => fail!("Table column fragments do not need to split"),
            UnscannedTextFragment(_) => fail!("Unscanned text fragments should have been scanned by now!"),
            ScannedTextFragment(ref text_fragment_info) => {
                let mut new_line_pos = self.new_line_pos.clone();
                let cur_new_line_pos = new_line_pos.shift().unwrap();

                let left_range = Range::new(text_fragment_info.range.begin(), cur_new_line_pos);
                let right_range = Range::new(text_fragment_info.range.begin() + cur_new_line_pos + CharIndex(1),
                                             text_fragment_info.range.length() - (cur_new_line_pos + CharIndex(1)));

                // Left fragment is for left text of first founded new-line character.
                let left_fragment = SplitInfo::new(left_range, text_fragment_info);

                // Right fragment is for right text of first founded new-line character.
                let right_fragment = if right_range.length() > CharIndex(0) {
                    Some(SplitInfo::new(right_range, text_fragment_info))
                } else {
                    None
                };

                Some((left_fragment, right_fragment, text_fragment_info.run.clone()))
            }
        }
    }

    /// Attempts to find the split positions of a text fragment so that its width is
    /// no more than `max_width`.
    ///
    /// A return value of `None` indicates that the fragment could not be split.
    /// Otherwise the information pertaining to the split is returned. The left
    /// and right split information are both optional due to the possibility of
    /// them being whitespace.
    //
    // TODO(bjz): The text run should be removed in the future, but it is currently needed for
    // the current method of fragment splitting in the `inline::try_append_*` functions.
    pub fn find_split_info_for_width(&self, start: CharIndex, max_width: Au, starts_line: bool)
            -> Option<(Option<SplitInfo>, Option<SplitInfo>, Arc<Box<TextRun>> /* TODO(bjz): remove */)> {
        match self.specific {
            GenericFragment | IframeFragment(_) | ImageFragment(_) | TableFragment | TableCellFragment |
            TableRowFragment | TableWrapperFragment => None,
            TableColumnFragment(_) => fail!("Table column fragments do not have width"),
            UnscannedTextFragment(_) => fail!("Unscanned text fragments should have been scanned by now!"),
            ScannedTextFragment(ref text_fragment_info) => {
                let mut pieces_processed_count: uint = 0;
                let mut remaining_width: Au = max_width;
                let mut left_range = Range::new(text_fragment_info.range.begin() + start, CharIndex(0));
                let mut right_range: Option<Range<CharIndex>> = None;

                debug!("split_to_width: splitting text fragment (strlen={}, range={}, avail_width={})",
                       text_fragment_info.run.text.len(),
                       text_fragment_info.range,
                       max_width);

                for (glyphs, offset, slice_range) in text_fragment_info.run.iter_slices_for_range(
                        &text_fragment_info.range) {
                    debug!("split_to_width: considering slice (offset={}, range={}, \
                                                               remain_width={})",
                           offset,
                           slice_range,
                           remaining_width);

                    let metrics = text_fragment_info.run.metrics_for_slice(glyphs, &slice_range);
                    let advance = metrics.advance_width;

                    let should_continue;
                    if advance <= remaining_width {
                        should_continue = true;

                        if starts_line && pieces_processed_count == 0 && glyphs.is_whitespace() {
                            debug!("split_to_width: case=skipping leading trimmable whitespace");
                            left_range.shift_by(slice_range.length());
                        } else {
                            debug!("split_to_width: case=enlarging span");
                            remaining_width = remaining_width - advance;
                            left_range.extend_by(slice_range.length());
                        }
                    } else {
                        // The advance is more than the remaining width.
                        should_continue = false;
                        let slice_begin = offset + slice_range.begin();
                        let slice_end = offset + slice_range.end();

                        if glyphs.is_whitespace() {
                            // If there are still things after the trimmable whitespace, create the
                            // right chunk.
                            if slice_end < text_fragment_info.range.end() {
                                debug!("split_to_width: case=skipping trimmable trailing \
                                        whitespace, then split remainder");
                                let right_range_end = text_fragment_info.range.end() - slice_end;
                                right_range = Some(Range::new(slice_end, right_range_end));
                            } else {
                                debug!("split_to_width: case=skipping trimmable trailing \
                                        whitespace");
                            }
                        } else if slice_begin < text_fragment_info.range.end() {
                            // There are still some things left over at the end of the line. Create
                            // the right chunk.
                            let right_range_end = text_fragment_info.range.end() - slice_begin;
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

                let left_is_some = left_range.length() > CharIndex(0);

                if (pieces_processed_count == 1 || !left_is_some) && !starts_line {
                    None
                } else {
                    let left = if left_is_some {
                        Some(SplitInfo::new(left_range, text_fragment_info))
                    } else {
                         None
                    };
                    let right = right_range.map(|right_range| SplitInfo::new(right_range, text_fragment_info));

                    Some((left, right, text_fragment_info.run.clone()))
                }
            }
        }
    }

    /// Returns true if this fragment is an unscanned text fragment that consists entirely of whitespace.
    pub fn is_whitespace_only(&self) -> bool {
        match self.specific {
            UnscannedTextFragment(ref text_fragment_info) => is_whitespace(text_fragment_info.text),
            _ => false,
        }
    }

    /// Assigns replaced width, padding, and margins for this fragment only if it is replaced
    /// content per CSS 2.1 § 10.3.2.
    pub fn assign_replaced_width_if_necessary(&mut self,
                                              container_width: Au,
                                              inline_fragment_context:
                                                Option<InlineFragmentContext>) {
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment | TableRowFragment |
            TableWrapperFragment => return,
            TableColumnFragment(_) => fail!("Table column fragments do not have width"),
            UnscannedTextFragment(_) => fail!("Unscanned text fragments should have been scanned by now!"),
            ImageFragment(_) | ScannedTextFragment(_) => {}
        };

        self.compute_border_padding_margins(container_width, inline_fragment_context);

        let style_width = self.style().get_box().width;
        let style_height = self.style().get_box().height;
        let noncontent_width = self.border_padding.horizontal();

        match self.specific {
            ScannedTextFragment(_) => {
                // Scanned text fragments will have already had their content widths assigned by this
                // point.
                self.border_box.size.width = self.border_box.size.width + noncontent_width
            }
            ImageFragment(ref mut image_fragment_info) => {
                // TODO(ksh8281): compute border,margin
                let width = ImageFragmentInfo::style_length(style_width,
                                                       image_fragment_info.dom_width,
                                                       container_width);
                let height = ImageFragmentInfo::style_length(style_height,
                                                        image_fragment_info.dom_height,
                                                        Au(0));

                let width = match (width,height) {
                    (Auto, Auto) => image_fragment_info.image_width(),
                    (Auto,Specified(h)) => {
                        let scale = image_fragment_info.
                            image_height().to_f32().unwrap() / h.to_f32().unwrap();
                        Au::new((image_fragment_info.image_width().to_f32().unwrap() / scale) as i32)
                    },
                    (Specified(w), _) => w,
                };

                self.border_box.size.width = width + noncontent_width;
                image_fragment_info.computed_width = Some(width);
            }
            _ => fail!("this case should have been handled above"),
        }
    }

    /// Assign height for this fragment if it is replaced content. The width must have been assigned
    /// first.
    ///
    /// Ideally, this should follow CSS 2.1 § 10.6.2.
    pub fn assign_replaced_height_if_necessary(&mut self) {
        match self.specific {
            GenericFragment | IframeFragment(_) | TableFragment | TableCellFragment | TableRowFragment |
            TableWrapperFragment => return,
            TableColumnFragment(_) => fail!("Table column fragments do not have height"),
            UnscannedTextFragment(_) => fail!("Unscanned text fragments should have been scanned by now!"),
            ImageFragment(_) | ScannedTextFragment(_) => {}
        }

        let style_width = self.style().get_box().width;
        let style_height = self.style().get_box().height;
        let noncontent_height = self.border_padding.vertical();

        match self.specific {
            ImageFragment(ref mut image_fragment_info) => {
                // TODO(ksh8281): compute border,margin,padding
                let width = image_fragment_info.computed_width();
                // FIXME(ksh8281): we shouldn't assign height this way
                // we don't know about size of parent's height
                let height = ImageFragmentInfo::style_length(style_height,
                                                        image_fragment_info.dom_height,
                                                        Au(0));

                let height = match (style_width, image_fragment_info.dom_width, height) {
                    (LPA_Auto, None, Auto) => {
                        image_fragment_info.image_height()
                    },
                    (_,_,Auto) => {
                        let scale = image_fragment_info.image_width().to_f32().unwrap()
                            / width.to_f32().unwrap();
                        Au::new((image_fragment_info.image_height().to_f32().unwrap() / scale) as i32)
                    },
                    (_,_,Specified(h)) => {
                        h
                    }
                };

                image_fragment_info.computed_height = Some(height);
                self.border_box.size.height = height + noncontent_height
            }
            ScannedTextFragment(_) => {
                // Scanned text fragments' content heights are calculated by the text run scanner
                // during flow construction.
                self.border_box.size.height = self.border_box.size.height + noncontent_height
            }
            _ => fail!("should have been handled above"),
        }
    }

    /// Calculates height above baseline, depth below baseline, and ascent for this fragment when
    /// used in an inline formatting context. See CSS 2.1 § 10.8.1.
    pub fn inline_metrics(&self) -> InlineMetrics {
        match self.specific {
            ImageFragment(ref image_fragment_info) => {
                let computed_height = image_fragment_info.computed_height();
                InlineMetrics {
                    height_above_baseline: computed_height + self.border_padding.vertical(),
                    depth_below_baseline: Au(0),
                    ascent: computed_height + self.border_padding.bottom,
                }
            }
            ScannedTextFragment(ref text_fragment) => {
                // See CSS 2.1 § 10.8.1.
                let font_size = self.style().get_font().font_size;
                let line_height = self.calculate_line_height(font_size);
                InlineMetrics::from_font_metrics(&text_fragment.run.font_metrics, line_height)
            }
            _ => {
                InlineMetrics {
                    height_above_baseline: self.border_box.size.height,
                    depth_below_baseline: Au(0),
                    ascent: self.border_box.size.height,
                }
            }
        }
    }

    /// Returns true if this fragment can merge with another adjacent fragment or false otherwise.
    pub fn can_merge_with_fragment(&self, other: &Fragment) -> bool {
        match (&self.specific, &other.specific) {
            (&UnscannedTextFragment(_), &UnscannedTextFragment(_)) => {
                self.font_style() == other.font_style() &&
                    self.text_decoration() == other.text_decoration()
            }
            _ => false,
        }
    }

    /// Returns true if the contents should be clipped (i.e. if `overflow` is `hidden`).
    pub fn needs_clip(&self) -> bool {
        self.style().get_box().overflow == overflow::hidden
    }

    /// A helper function to return a debug string describing the side offsets for one of the rect
    /// box model properties (border, padding, or margin).
    fn side_offsets_debug_fmt(&self, name: &str,
                              value: SideOffsets2D<Au>,
                              f: &mut fmt::Formatter) -> fmt::Result {
        if value.is_zero() {
            Ok(())
        } else {
            write!(f.buf, "{}{},{},{},{}",
                name,
                value.top,
                value.right,
                value.bottom,
                value.left)
        }
    }

    /// Sends the size and position of this iframe fragment to the constellation. This is out of
    /// line to guide inlining.
    #[inline(never)]
    fn finalize_position_and_size_of_iframe(&self,
                                            iframe_fragment: &IframeFragmentInfo,
                                            offset: Point2D<Au>,
                                            layout_context: &LayoutContext) {
        let left = offset.x + self.margin.left + self.border_padding.left;
        let top = offset.y + self.margin.top + self.border_padding.top;
        let width = self.content_box().size.width;
        let height = self.content_box().size.height;
        let origin = Point2D(geometry::to_frac_px(left) as f32, geometry::to_frac_px(top) as f32);
        let size = Size2D(geometry::to_frac_px(width) as f32, geometry::to_frac_px(height) as f32);
        let rect = Rect(origin, size);

        debug!("finalizing position and size of iframe for {:?},{:?}",
               iframe_fragment.pipeline_id,
               iframe_fragment.subpage_id);
        let msg = FrameRectMsg(iframe_fragment.pipeline_id, iframe_fragment.subpage_id, rect);
        let ConstellationChan(ref chan) = layout_context.constellation_chan;
        chan.send(msg)
    }
}

impl fmt::Show for Fragment {
    /// Outputs a debugging string describing this fragment.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f.buf, "({} ",
            match self.specific {
                GenericFragment => "GenericFragment",
                IframeFragment(_) => "IframeFragment",
                ImageFragment(_) => "ImageFragment",
                ScannedTextFragment(_) => "ScannedTextFragment",
                TableFragment => "TableFragment",
                TableCellFragment => "TableCellFragment",
                TableColumnFragment(_) => "TableColumnFragment",
                TableRowFragment => "TableRowFragment",
                TableWrapperFragment => "TableWrapperFragment",
                UnscannedTextFragment(_) => "UnscannedTextFragment",
        }));
        try!(self.side_offsets_debug_fmt("bp", self.border_padding, f));
        try!(write!(f.buf, " "));
        try!(self.side_offsets_debug_fmt("m", self.margin, f));
        write!(f.buf, ")")
    }
}

/// An object that accumulates display lists of child flows, applying a clipping rect if necessary.
pub struct ChildDisplayListAccumulator {
    clip_display_item: Option<Box<ClipDisplayItem>>,
}

impl ChildDisplayListAccumulator {
    /// Creates a `ChildDisplayListAccumulator` from the `overflow` property in the given style.
    fn new(style: &ComputedValues, bounds: Rect<Au>, node: OpaqueNode, level: StackingLevel)
           -> ChildDisplayListAccumulator {
        ChildDisplayListAccumulator {
            clip_display_item: match style.get_box().overflow {
                overflow::hidden => {
                    Some(box ClipDisplayItem {
                        base: BaseDisplayItem::new(bounds, node, level),
                        children: DisplayList::new(),
                    })
                }
                _ => None,
            }
        }
    }

    /// Pushes the given display item onto this display list.
    pub fn push(&mut self, parent_display_list: &mut DisplayList, item: DisplayItem) {
        match self.clip_display_item {
            None => parent_display_list.push(item),
            Some(ref mut clip_display_item) => clip_display_item.children.push(item),
        }
    }

    /// Pushes the display items from the given child onto this display list.
    pub fn push_child(&mut self, parent_display_list: &mut DisplayList, child: &mut Flow) {
        let kid_display_list = mem::replace(&mut flow::mut_base(child).display_list,
                                            DisplayList::new());
        match self.clip_display_item {
            None => parent_display_list.push_all_move(kid_display_list),
            Some(ref mut clip_display_item) => {
                clip_display_item.children.push_all_move(kid_display_list)
            }
        }
    }

    /// Consumes this accumulator and pushes the clipping item, if any, onto the display list
    /// associated with the given flow, along with the items in the given display list.
    pub fn finish(self, parent: &mut Flow, mut display_list: DisplayList) {
        let ChildDisplayListAccumulator {
            clip_display_item
        } = self;
        match clip_display_item {
            None => {}
            Some(clip_display_item) => display_list.push(ClipDisplayItemClass(clip_display_item)),
        }
        flow::mut_base(parent).display_list = display_list
    }
}

