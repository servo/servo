/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A list of common mouse cursors per CSS3-UI § 8.1.1.

#[deriving(Clone, PartialEq, FromPrimitive, Show)]
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

