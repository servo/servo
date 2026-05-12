/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};

use app_units::Au;
use atomic_refcell::AtomicRef;
use bitflags::bitflags;
use euclid::{Point2D, Rect, Size2D};
use layout_api::{LayoutElement, LayoutNode, PseudoElementChain, combine_id_with_fragment_type};
use malloc_size_of::malloc_size_of_is_0;
use malloc_size_of_derive::MallocSizeOf;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use script::layout_dom::ServoLayoutNode;
use servo_arc::Arc as ServoArc;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;
use style::selector_parser::PseudoElement;
use style_traits::CSSPixel;
use web_atoms::local_name;

use crate::SharedStyle;
use crate::dom_traversal::NodeAndStyleInfo;
use crate::geom::{PhysicalPoint, PhysicalRect, PhysicalSize};

#[derive(Clone, Debug, Default, MallocSizeOf, FromPrimitive)]
#[repr(u8)]
pub(crate) enum FragmentStatus {
    /// This is a brand new fragment.
    #[default]
    New = 0,
    /// The style of the fragment has changed.
    StyleChanged = 1,
    /// The fragment was reused between layouts, but its final layout
    /// position may have changed.
    PositionMaybeChanged = 2,
    /// The fragment hasn't changed.
    Clean = 3,
}

/// This data structure stores fields that are common to all non-base
/// Fragment types and should generally be the first member of all
/// concrete fragments.
#[derive(MallocSizeOf)]
pub(crate) struct BaseFragment {
    /// A tag which identifies the DOM node and pseudo element of this
    /// Fragment's content. If this fragment is for an anonymous box,
    /// the tag will be None.
    pub tag: Option<Tag>,

    /// Flags which various information about this fragment used during
    /// layout.
    pub flags: FragmentFlags,

    /// The style for this [`BaseFragment`]. Depending on the fragment type this is either
    /// a shared or non-shared style.
    pub style: SharedStyle,

    /// The content rect of this fragment in the parent fragment's content rectangle. This
    /// does not include padding, border, or margin -- it only includes content. This is
    /// relative to the parent containing block.
    rect: Rect<AtomicI32, CSSPixel>,

    /// A [`FragmentStatus`] used to track fragment reuse when collecting reflow statistics.
    pub status: AtomicU8,
}

impl std::fmt::Debug for BaseFragment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseFragment")
            .field("tag", &self.tag)
            .field("flags", &self.flags)
            .field("rect", &self.rect)
            .field(
                "status",
                &FragmentStatus::from_u8(self.status.load(Ordering::Relaxed)),
            )
            .finish()
    }
}

impl BaseFragment {
    pub(crate) fn new(
        base_fragment_info: BaseFragmentInfo,
        style: SharedStyle,
        rect: PhysicalRect<Au>,
    ) -> Self {
        Self {
            tag: base_fragment_info.tag,
            flags: base_fragment_info.flags,
            style,
            rect: Rect::new(
                Point2D::new(rect.origin.x.0.into(), rect.origin.y.0.into()),
                Size2D::new(rect.size.width.0.into(), rect.size.height.0.into()),
            ),
            status: AtomicU8::new(FragmentStatus::New as u8),
        }
    }

    #[inline]
    pub(crate) fn rect(&self) -> PhysicalRect<Au> {
        PhysicalRect::new(
            Point2D::new(
                Au::new(self.rect.origin.x.load(Ordering::Relaxed)),
                Au::new(self.rect.origin.y.load(Ordering::Relaxed)),
            ),
            Size2D::new(
                Au::new(self.rect.size.width.load(Ordering::Relaxed)),
                Au::new(self.rect.size.height.load(Ordering::Relaxed)),
            ),
        )
    }

    #[inline]
    pub(crate) fn set_rect(&self, new_rect: PhysicalRect<Au>) {
        let origin = &self.rect.origin;
        origin.x.store(new_rect.origin.x.0, Ordering::Relaxed);
        origin.y.store(new_rect.origin.y.0, Ordering::Relaxed);

        let size = &self.rect.size;
        size.width.store(new_rect.size.width.0, Ordering::Relaxed);
        size.height.store(new_rect.size.height.0, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn translate_rect(&self, offset: PhysicalSize<Au>) {
        // This code explicitly does not use `AtomicI32::fetch_add`, as we rely on Au's
        // overflow detection to clamp the resulting value between `MAX_AU` and `MIN_AU`.
        let origin = &self.rect.origin;
        let new_x = Au::new(origin.x.load(Ordering::Relaxed)) + offset.width;
        origin.x.store(new_x.0, Ordering::Relaxed);
        let new_y = Au::new(origin.y.load(Ordering::Relaxed)) + offset.height;
        origin.y.store(new_y.0, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn set_rect_origin(&self, offset: PhysicalPoint<Au>) {
        let origin = &self.rect.origin;
        origin.x.store(offset.x.0, Ordering::Relaxed);
        origin.y.store(offset.y.0, Ordering::Relaxed);
    }

    pub(crate) fn is_anonymous(&self) -> bool {
        self.tag.is_none()
    }

    pub(crate) fn status(&self) -> FragmentStatus {
        FragmentStatus::from_u8(self.status.load(Ordering::Relaxed))
            .expect("Unknown FragmentStatus value")
    }

    pub(crate) fn set_status(&self, new_status: FragmentStatus) {
        self.status.store(new_status as u8, Ordering::Relaxed)
    }

    pub(crate) fn repair_style(&self, style: &ServoArc<ComputedValues>) {
        *self.style.borrow_mut() = style.clone();
        self.set_status(FragmentStatus::StyleChanged);
    }

    pub(crate) fn style<'a>(&'a self) -> AtomicRef<'a, ServoArc<ComputedValues>> {
        self.style.borrow()
    }
}

/// Information necessary to construct a new BaseFragment.
#[derive(Clone, Copy, Debug, MallocSizeOf)]
pub(crate) struct BaseFragmentInfo {
    /// The tag to use for the new BaseFragment, if it is not an anonymous Fragment.
    pub tag: Option<Tag>,

    /// The flags to use for the new BaseFragment.
    pub flags: FragmentFlags,
}

impl BaseFragmentInfo {
    pub(crate) fn anonymous() -> Self {
        Self {
            tag: None,
            flags: FragmentFlags::empty(),
        }
    }

    pub(crate) fn new_for_testing(id: usize) -> Self {
        Self {
            tag: Some(Tag {
                node: OpaqueNode(id),
                pseudo_element_chain: Default::default(),
            }),
            flags: FragmentFlags::empty(),
        }
    }

    pub(crate) fn is_anonymous(&self) -> bool {
        self.tag.is_none()
    }
}

impl From<&NodeAndStyleInfo<'_>> for BaseFragmentInfo {
    fn from(info: &NodeAndStyleInfo) -> Self {
        info.node.into()
    }
}

impl From<ServoLayoutNode<'_>> for BaseFragmentInfo {
    fn from(node: ServoLayoutNode) -> Self {
        let pseudo_element_chain = node.pseudo_element_chain();
        let mut flags = FragmentFlags::empty();

        // Anonymous boxes should not have a tag, because they should not take part in hit testing.
        //
        // TODO(mrobinson): It seems that anonymous boxes should take part in hit testing in some
        // cases, but currently this means that the order of hit test results isn't as expected for
        // some WPT tests. This needs more investigation.
        if matches!(
            pseudo_element_chain.innermost(),
            Some(PseudoElement::ServoAnonymousBox) |
                Some(PseudoElement::ServoAnonymousTable) |
                Some(PseudoElement::ServoAnonymousTableCell) |
                Some(PseudoElement::ServoAnonymousTableRow)
        ) {
            return Self::anonymous();
        }

        if let Some(element) = node.as_html_element() {
            if element.is_body_element_of_html_element_root() {
                flags.insert(FragmentFlags::IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT);
            }

            match element.local_name() {
                &local_name!("br") => {
                    flags.insert(FragmentFlags::IS_BR_ELEMENT);
                },
                &local_name!("table") | &local_name!("th") | &local_name!("td") => {
                    flags.insert(FragmentFlags::IS_TABLE_TH_OR_TD_ELEMENT);
                },
                &local_name!("input") => {
                    flags.insert(FragmentFlags::IS_INPUT_ELEMENT);
                },
                _ => {},
            }

            if element.is_root() {
                flags.insert(FragmentFlags::IS_ROOT_ELEMENT);
            }
        };

        Self {
            tag: Some(node.into()),
            flags,
        }
    }
}

bitflags! {
    /// Flags used to track various information about a DOM node during layout.
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct FragmentFlags: u16 {
        /// Whether or not the node that created this fragment is a `<body>` element on an HTML document.
        const IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT = 1 << 0;
        /// Whether or not the node that created this Fragment is a `<br>` element.
        const IS_BR_ELEMENT = 1 << 1;
        /// Whether or not the node that created this Fragment is a widget. Widgets behave similarly to
        /// replaced elements, e.g. they are atomic when inline-level, and their automatic inline size
        /// doesn't stretch when block-level.
        /// <https://drafts.csswg.org/css-ui/#widget>
        const IS_WIDGET = 1 << 2;
        /// Whether or not this Fragment is a flex item or a grid item.
        const IS_FLEX_OR_GRID_ITEM = 1 << 3;
        /// Whether or not this Fragment was created to contain a replaced element or is
        /// a replaced element.
        const IS_REPLACED = 1 << 4;
        /// Whether or not the node that created was a `<table>`, `<th>` or
        /// `<td>` element. Note that this does *not* include elements with
        /// `display: table` or `display: table-cell`.
        const IS_TABLE_TH_OR_TD_ELEMENT = 1 << 5;
        /// Whether or not this Fragment was created to contain a list item marker
        /// with a used value of `list-style-position: outside`.
        const IS_OUTSIDE_LIST_ITEM_MARKER = 1 << 6;
        /// Avoid painting the borders, backgrounds, and drop shadow for this fragment, this is used
        /// for empty table cells when 'empty-cells' is 'hide' and also table wrappers.  This flag
        /// doesn't avoid hit-testing nor does it prevent the painting outlines.
        const DO_NOT_PAINT = 1 << 7;
        /// Whether or not the size of this fragment depends on the block size of its container
        /// and the fragment can be a flex item. This flag is used to cache items during flex
        /// layout.
        const SIZE_DEPENDS_ON_BLOCK_CONSTRAINTS_AND_CAN_BE_CHILD_OF_FLEX_ITEM = 1 << 8;
        /// Whether or not the node that created this fragment is the root element.
        const IS_ROOT_ELEMENT = 1 << 9;
        /// If element has propagated the overflow value to viewport.
        const PROPAGATED_OVERFLOW_TO_VIEWPORT = 1 << 10;
        /// Whether or not this is a table cell that is part of a collapsed row or column.
        /// In that case it should not be painted.
        const IS_COLLAPSED = 1 << 11;
        /// Whether or not the node that created this Fragment is a `<input>` element.
        const IS_INPUT_ELEMENT = 1 << 12;

    }
}

malloc_size_of_is_0!(FragmentFlags);

/// A data structure used to hold DOM and pseudo-element information about
/// a particular layout object.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq)]
pub(crate) struct Tag {
    pub(crate) node: OpaqueNode,
    pub(crate) pseudo_element_chain: PseudoElementChain,
}

impl Tag {
    pub(crate) fn to_display_list_fragment_id(self) -> u64 {
        combine_id_with_fragment_type(self.node.id(), self.pseudo_element_chain.primary.into())
    }
}

impl From<ServoLayoutNode<'_>> for Tag {
    fn from(node: ServoLayoutNode<'_>) -> Self {
        Self {
            node: node.opaque(),
            pseudo_element_chain: node.pseudo_element_chain(),
        }
    }
}
