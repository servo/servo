/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bitflags::bitflags;
use script_layout_interface::{combine_id_with_fragment_type, FragmentType};
use serde::Serialize;
use style::dom::OpaqueNode;
use style::selector_parser::PseudoElement;

use crate::layout_debug::DebugId;

/// This data structure stores fields that are common to all non-base
/// Fragment types and should generally be the first member of all
/// concrete fragments.
#[derive(Debug, Serialize)]
pub(crate) struct BaseFragment {
    /// A tag which identifies the DOM node and pseudo element of this
    /// Fragment's content. If this fragment isn't related to any DOM
    /// node at all, the tag will be None.
    pub tag: Option<Tag>,

    /// An id used to uniquely identify this Fragment in debug builds.
    pub debug_id: DebugId,

    /// Flags which various information about this fragment used during
    /// layout.
    pub flags: FragmentFlags,
}

impl BaseFragment {
    pub(crate) fn anonymous() -> Self {
        BaseFragment {
            tag: None,
            debug_id: DebugId::new(),
            flags: FragmentFlags::empty(),
        }
    }

    /// Returns true if this fragment is non-anonymous and it is for the given
    /// OpaqueNode, regardless of the pseudo element.
    pub(crate) fn is_for_node(&self, node: OpaqueNode) -> bool {
        self.tag.map(|tag| tag.node == node).unwrap_or(false)
    }
}

/// Information necessary to construct a new BaseFragment.
#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct BaseFragmentInfo {
    /// The tag to use for the new BaseFragment, if it is not an anonymous Fragment.
    pub tag: Option<Tag>,

    /// The flags to use for the new BaseFragment.
    pub flags: FragmentFlags,
}

impl BaseFragmentInfo {
    pub(crate) fn new_for_node(node: OpaqueNode) -> Self {
        Self {
            tag: Some(Tag::new(node)),
            flags: FragmentFlags::empty(),
        }
    }

    pub(crate) fn anonymous() -> Self {
        Self {
            tag: None,
            flags: FragmentFlags::empty(),
        }
    }
}

impl From<BaseFragmentInfo> for BaseFragment {
    fn from(info: BaseFragmentInfo) -> Self {
        Self {
            tag: info.tag,
            debug_id: DebugId::new(),
            flags: info.flags,
        }
    }
}

bitflags! {
    /// Flags used to track various information about a DOM node during layout.
    #[derive(Clone, Copy, Debug, Serialize)]
    pub(crate) struct FragmentFlags: u8 {
        /// Whether or not the node that created this fragment is a `<body>` element on an HTML document.
        const IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT = 1 << 0;
        /// Whether or not the node that created this Fragment is a `<br>` element.
        const IS_BR_ELEMENT = 1 << 1;
        /// Whether or not the node that created was a `<table>`, `<th>` or
        /// `<td>` element. Note that this does *not* include elements with
        /// `display: table` or `display: table-cell`.
        const IS_TABLE_TH_OR_TD_ELEMENT = 1 << 2;
        /// Whether or not this Fragment was created to contain a replaced element or is
        /// a replaced element.
        const IS_REPLACED = 1 << 3;
        /// Whether or not this Fragment was created to contain a list item marker
        /// with a used value of `list-style-position: outside`.
        const IS_OUTSIDE_LIST_ITEM_MARKER = 1 << 4;
        /// Avoid painting the borders, backgrounds, and drop shadow for this fragment, this is used
        /// for empty table cells when 'empty-cells' is 'hide' and also table wrappers.  This flag
        /// doesn't avoid hit-testing nor does it prevent the painting outlines.
        const DO_NOT_PAINT = 1 << 5;
    }
}

/// A data structure used to hold DOM and pseudo-element information about
/// a particular layout object.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct Tag {
    pub(crate) node: OpaqueNode,
    pub(crate) pseudo: Option<PseudoElement>,
}

impl Tag {
    /// Create a new Tag for a non-pseudo element. This is mainly used for
    /// matching existing tags, since it does not accept an `info` argument.
    pub(crate) fn new(node: OpaqueNode) -> Self {
        Tag { node, pseudo: None }
    }

    /// Create a new Tag for a pseudo element. This is mainly used for
    /// matching existing tags, since it does not accept an `info` argument.
    pub(crate) fn new_pseudo(node: OpaqueNode, pseudo: Option<PseudoElement>) -> Self {
        Tag { node, pseudo }
    }

    /// Returns true if this tag is for a pseudo element.
    pub(crate) fn is_pseudo(&self) -> bool {
        self.pseudo.is_some()
    }

    pub(crate) fn to_display_list_fragment_id(self) -> u64 {
        let fragment_type = match self.pseudo {
            Some(PseudoElement::Before) => FragmentType::BeforePseudoContent,
            Some(PseudoElement::After) => FragmentType::AfterPseudoContent,
            _ => FragmentType::FragmentBody,
        };
        combine_id_with_fragment_type(self.node.id(), fragment_type)
    }
}
