/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A list of common mouse cursors per CSS3-UI § 8.1.1.

use std::fmt;

#[deriving(Clone, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum Cursor {
    NoCursor = 0,
    DefaultCursor = 1,
    PointerCursor = 2,
    ContextMenuCursor = 3,
    HelpCursor = 4,
    ProgressCursor = 5,
    WaitCursor = 6,
    CellCursor = 7,
    CrosshairCursor = 8,
    TextCursor = 9,
    VerticalTextCursor = 10,
    AliasCursor = 11,
    CopyCursor = 12,
    MoveCursor = 13,
    NoDropCursor = 14,
    NotAllowedCursor = 15,
    GrabCursor = 16,
    GrabbingCursor = 17,
    EResizeCursor = 18,
    NResizeCursor = 19,
    NeResizeCursor = 20,
    NwResizeCursor = 21,
    SResizeCursor = 22,
    SeResizeCursor = 23,
    SwResizeCursor = 24,
    WResizeCursor = 25,
    EwResizeCursor = 26,
    NsResizeCursor = 27,
    NeswResizeCursor = 28,
    NwseResizeCursor = 29,
    ColResizeCursor = 30,
    RowResizeCursor = 31,
    AllScrollCursor = 32,
    ZoomInCursor = 33,
    ZoomOutCursor = 34,
}

impl fmt::Show for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Cursor::NoCursor => write!(f, "none"),
            Cursor::DefaultCursor => write!(f, "default"),
            Cursor::PointerCursor => write!(f, "pointer"),
            Cursor::ContextMenuCursor => write!(f, "context-menu"),
            Cursor::HelpCursor => write!(f, "help"),
            Cursor::ProgressCursor => write!(f, "progress"),
            Cursor::WaitCursor => write!(f, "wait"),
            Cursor::CellCursor => write!(f, "cell"),
            Cursor::CrosshairCursor => write!(f, "crosshair"),
            Cursor::TextCursor => write!(f, "text"),
            Cursor::VerticalTextCursor => write!(f, "vertical-text"),
            Cursor::AliasCursor => write!(f, "alias"),
            Cursor::CopyCursor => write!(f, "copy"),
            Cursor::MoveCursor => write!(f, "move"),
            Cursor::NoDropCursor => write!(f, "no-drop"),
            Cursor::NotAllowedCursor => write!(f, "not-allowed"),
            Cursor::GrabCursor => write!(f, "grab"),
            Cursor::GrabbingCursor => write!(f, "grabbing"),
            Cursor::EResizeCursor => write!(f, "e-resize"),
            Cursor::NResizeCursor => write!(f, "n-resize"),
            Cursor::NeResizeCursor => write!(f, "ne-resize"),
            Cursor::NwResizeCursor => write!(f, "nw-resize"),
            Cursor::SResizeCursor => write!(f, "s-resize"),
            Cursor::SwResizeCursor => write!(f, "sw-resize"),
            Cursor::SeResizeCursor => write!(f, "se-resize"),
            Cursor::WResizeCursor => write!(f, "w-resize"),
            Cursor::EwResizeCursor => write!(f, "ew-resize"),
            Cursor::NsResizeCursor => write!(f, "ns-resize"),
            Cursor::NeswResizeCursor => write!(f, "nesw-resize"),
            Cursor::NwseResizeCursor => write!(f, "nwse-resize"),
            Cursor::ColResizeCursor => write!(f, "col-resize"),
            Cursor::RowResizeCursor => write!(f, "row-resize"),
            Cursor::AllScrollCursor => write!(f, "all-scroll"),
            Cursor::ZoomInCursor => write!(f, "zoom-in"),
            Cursor::ZoomOutCursor => write!(f, "zoom-out"),
        }
    }
}
