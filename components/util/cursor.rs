/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A list of common mouse cursors per CSS3-UI § 8.1.1.

use cssparser::ToCss;
use std::ascii::AsciiExt;
use text_writer::TextWriter;

macro_rules! define_cursor {
    ($( $css: expr => $variant: ident = $value: expr, )+) => {
        #[derive(Clone, Copy, PartialEq, Eq, FromPrimitive, Debug)]
        #[repr(u8)]
        pub enum Cursor {
            $( $variant = $value ),+
        }

        impl Cursor {
            pub fn from_css_keyword(keyword: &str) -> Result<Cursor, ()> {
                match_ignore_ascii_case! { keyword,
                    $( concat!($css) => Ok(Cursor::$variant) ),+
                    _ => Err(())
                }
            }
        }

        impl ToCss for Cursor {
            fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result where W: TextWriter {
                match self {
                    $( &Cursor::$variant => dest.write_str($css) ),+
                }
            }
        }
    }
}


define_cursor! {
    "none" => NoCursor = 0,
    "default" => DefaultCursor = 1,
    "pointer" => PointerCursor = 2,
    "context-menu" => ContextMenuCursor = 3,
    "help" => HelpCursor = 4,
    "progress" => ProgressCursor = 5,
    "wait" => WaitCursor = 6,
    "cell" => CellCursor = 7,
    "crosshair" => CrosshairCursor = 8,
    "text" => TextCursor = 9,
    "vertical-text" => VerticalTextCursor = 10,
    "alias" => AliasCursor = 11,
    "copy" => CopyCursor = 12,
    "move" => MoveCursor = 13,
    "no-drop" => NoDropCursor = 14,
    "not-allowed" => NotAllowedCursor = 15,
    "grab" => GrabCursor = 16,
    "grabbing" => GrabbingCursor = 17,
    "e-resize" => EResizeCursor = 18,
    "n-resize" => NResizeCursor = 19,
    "ne-resize" => NeResizeCursor = 20,
    "nw-resize" => NwResizeCursor = 21,
    "s-resize" => SResizeCursor = 22,
    "se-resize" => SeResizeCursor = 23,
    "sw-resize" => SwResizeCursor = 24,
    "w-resize" => WResizeCursor = 25,
    "ew-resize" => EwResizeCursor = 26,
    "ns-resize" => NsResizeCursor = 27,
    "nesw-resize" => NeswResizeCursor = 28,
    "nwse-resize" => NwseResizeCursor = 29,
    "col-resize" => ColResizeCursor = 30,
    "row-resize" => RowResizeCursor = 31,
    "all-scroll" => AllScrollCursor = 32,
    "zoom-in" => ZoomInCursor = 33,
    "zoom-out" => ZoomOutCursor = 34,
}
