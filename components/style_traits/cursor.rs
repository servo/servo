/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A list of common mouse cursors per CSS3-UI § 8.1.1.

use super::ToCss;

macro_rules! define_cursor {
    ($( $css: expr => $variant: ident = $value: expr, )+) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[cfg_attr(feature = "servo", derive(Deserialize, Serialize, HeapSizeOf))]
        #[repr(u8)]
        pub enum Cursor {
            $( $variant = $value ),+
        }

        impl Cursor {
            pub fn from_css_keyword(keyword: &str) -> Result<Cursor, ()> {
                match_ignore_ascii_case! { keyword,
                    $( concat!($css) => Ok(Cursor::$variant), )+
                    _ => Err(())
                }
            }
        }

        impl ToCss for Cursor {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result where W: ::std::fmt::Write {
                match *self {
                    $( Cursor::$variant => dest.write_str($css) ),+
                }
            }
        }
    }
}


define_cursor! {
    "none" => None = 0,
    "default" => Default = 1,
    "pointer" => Pointer = 2,
    "context-menu" => ContextMenu = 3,
    "help" => Help = 4,
    "progress" => Progress = 5,
    "wait" => Wait = 6,
    "cell" => Cell = 7,
    "crosshair" => Crosshair = 8,
    "text" => Text = 9,
    "vertical-text" => VerticalText = 10,
    "alias" => Alias = 11,
    "copy" => Copy = 12,
    "move" => Move = 13,
    "no-drop" => NoDrop = 14,
    "not-allowed" => NotAllowed = 15,
    "grab" => Grab = 16,
    "grabbing" => Grabbing = 17,
    "e-resize" => EResize = 18,
    "n-resize" => NResize = 19,
    "ne-resize" => NeResize = 20,
    "nw-resize" => NwResize = 21,
    "s-resize" => SResize = 22,
    "se-resize" => SeResize = 23,
    "sw-resize" => SwResize = 24,
    "w-resize" => WResize = 25,
    "ew-resize" => EwResize = 26,
    "ns-resize" => NsResize = 27,
    "nesw-resize" => NeswResize = 28,
    "nwse-resize" => NwseResize = 29,
    "col-resize" => ColResize = 30,
    "row-resize" => RowResize = 31,
    "all-scroll" => AllScroll = 32,
    "zoom-in" => ZoomIn = 33,
    "zoom-out" => ZoomOut = 34,
}
