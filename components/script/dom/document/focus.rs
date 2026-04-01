/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bitflags::bitflags;
use script_bindings::root::DomRoot;

use crate::dom::Node;

pub(crate) enum FocusOperation {
    Focus(FocusableArea),
    Unfocus,
}

/// The kind of focusable area a [`FocusableArea`] is. A [`FocusableArea`] may be click focusable,
/// sequentially focusable, or both.
#[derive(Clone, Copy, Debug, Default, MallocSizeOf)]
pub(crate) struct FocusableAreaKind(u8);

bitflags! {
    impl FocusableAreaKind: u8 {
        /// <https://html.spec.whatwg.org/multipage/#click-focusable>
        ///
        /// > A focusable area is said to be click focusable if the user agent determines that it is
        /// > click focusable. User agents should consider focusable areas with non-null tabindex values
        /// > to be click focusable.
        const Click = 1 << 0;
        /// <https://html.spec.whatwg.org/multipage/#sequentially-focusable>.
        ///
        /// > A focusable area is said to be sequentially focusable if it is included in its
        /// > Document's sequential focus navigation order and the user agent determines that it is
        /// > sequentially focusable.
        const Sequential = 1 << 1;
    }
}

pub(crate) enum FocusableArea {
    Node {
        node: DomRoot<Node>,
        kind: FocusableAreaKind,
    },
    Viewport,
}

impl FocusableArea {
    pub(crate) fn kind(&self) -> FocusableAreaKind {
        match self {
            FocusableArea::Node { kind, .. } => *kind,
            FocusableArea::Viewport => FocusableAreaKind::Click | FocusableAreaKind::Sequential,
        }
    }
}
