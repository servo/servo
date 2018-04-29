/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A list of common mouse cursors per CSS3-UI § 8.1.1.

use super::{CssWriter, KeywordsCollectFn, SpecifiedValueInfo, ToCss};

macro_rules! define_cursor {
    (
        common properties = [
            $( $c_css: expr => $c_variant: ident = $c_value: expr, )+
        ]
        gecko properties = [
            $( $g_css: expr => $g_variant: ident = $g_value: expr, )+
        ]
    ) => {
        /// <https://drafts.csswg.org/css-ui/#cursor>
        #[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq)]
        #[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
        #[repr(u8)]
        #[allow(missing_docs)]
        pub enum CursorKind {
            $( $c_variant = $c_value, )+
            $( #[cfg(feature = "gecko")] $g_variant = $g_value, )+
        }

        impl CursorKind {
            /// Given a CSS keyword, get the corresponding cursor enum.
            pub fn from_css_keyword(keyword: &str) -> Result<Self, ()> {
                match_ignore_ascii_case! { &keyword,
                    $( $c_css => Ok(CursorKind::$c_variant), )+
                    $( #[cfg(feature = "gecko")] $g_css => Ok(CursorKind::$g_variant), )+
                    _ => Err(())
                }
            }

            /// From the C u8 value, get the corresponding Cursor enum.
            pub fn from_u8(value: u8) -> Result<Self, ()> {
                match value {
                    $( $c_value => Ok(CursorKind::$c_variant), )+
                    $( #[cfg(feature = "gecko")] $g_value => Ok(CursorKind::$g_variant), )+
                    _ => Err(())
                }
            }
        }

        impl ToCss for CursorKind {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> ::std::fmt::Result where W: ::std::fmt::Write {
                match *self {
                    $(CursorKind::$c_variant => {
                        ::std::fmt::Write::write_str(dest, $c_css)
                    })+
                    $(#[cfg(feature = "gecko")] CursorKind::$g_variant => {
                        ::std::fmt::Write::write_str(dest, $g_css)
                    })+
                }
            }
        }

        impl SpecifiedValueInfo for CursorKind {
            fn collect_completion_keywords(f: KeywordsCollectFn) {
                f(&[
                    $($c_css,)+
                    $($g_css,)+
                ]);
            }
        }
    }
}

define_cursor! {
    common properties = [
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
        "auto" => Auto = 35,
    ]
    // gecko only properties
    gecko properties = [
        "-moz-grab" => MozGrab = 36,
        "-moz-grabbing" => MozGrabbing = 37,
        "-moz-zoom-in" => MozZoomIn = 38,
        "-moz-zoom-out" => MozZoomOut = 39,
    ]
}
