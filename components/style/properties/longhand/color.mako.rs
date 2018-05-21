/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Color", inherited=True) %>

<% from data import to_rust_ident %>

${helpers.predefined_type(
    "color",
    "ColorPropertyValue",
    "::cssparser::RGBA::new(0, 0, 0, 255)",
    animation_value_type="AnimatedRGBA",
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
    ignored_when_colors_disabled="True",
    spec="https://drafts.csswg.org/css-color/#color"
)}

// FIXME(#15973): Add servo support for system colors
//
// FIXME(emilio): Move outside of mako.
% if product == "gecko":
pub mod system_colors {
    <%
        # These are actually parsed. See nsCSSProps::kColorKTable
        system_colors = """activeborder activecaption appworkspace background buttonface
                           buttonhighlight buttonshadow buttontext captiontext graytext highlight
                           highlighttext inactiveborder inactivecaption inactivecaptiontext
                           infobackground infotext menu menutext scrollbar threeddarkshadow
                           threedface threedhighlight threedlightshadow threedshadow window
                           windowframe windowtext -moz-buttondefault -moz-buttonhoverface
                           -moz-buttonhovertext -moz-cellhighlight -moz-cellhighlighttext
                           -moz-eventreerow -moz-field -moz-fieldtext -moz-dialog -moz-dialogtext
                           -moz-dragtargetzone -moz-gtk-info-bar-text -moz-html-cellhighlight
                           -moz-html-cellhighlighttext -moz-mac-buttonactivetext
                           -moz-mac-chrome-active -moz-mac-chrome-inactive
                           -moz-mac-defaultbuttontext -moz-mac-focusring -moz-mac-menuselect
                           -moz-mac-menushadow -moz-mac-menutextdisable -moz-mac-menutextselect
                           -moz-mac-disabledtoolbartext -moz-mac-secondaryhighlight
                           -moz-mac-vibrancy-light -moz-mac-vibrancy-dark
                           -moz-mac-vibrant-titlebar-light -moz-mac-vibrant-titlebar-dark
                           -moz-mac-menupopup
                           -moz-mac-menuitem -moz-mac-active-menuitem -moz-mac-source-list
                           -moz-mac-source-list-selection -moz-mac-active-source-list-selection
                           -moz-mac-tooltip
                           -moz-menuhover -moz-menuhovertext -moz-menubartext -moz-menubarhovertext
                           -moz-oddtreerow -moz-win-mediatext -moz-win-communicationstext
                           -moz-win-accentcolor -moz-win-accentcolortext
                           -moz-nativehyperlinktext -moz-comboboxtext -moz-combobox""".split()

        # These are not parsed but must be serialized
        # They are only ever set directly by Gecko
        extra_colors = """WindowBackground WindowForeground WidgetBackground WidgetForeground
                          WidgetSelectBackground WidgetSelectForeground Widget3DHighlight Widget3DShadow
                          TextBackground TextForeground TextSelectBackground TextSelectForeground
                          TextSelectForegroundCustom TextSelectBackgroundDisabled TextSelectBackgroundAttention
                          TextHighlightBackground TextHighlightForeground IMERawInputBackground
                          IMERawInputForeground IMERawInputUnderline IMESelectedRawTextBackground
                          IMESelectedRawTextForeground IMESelectedRawTextUnderline
                          IMEConvertedTextBackground IMEConvertedTextForeground IMEConvertedTextUnderline
                          IMESelectedConvertedTextBackground IMESelectedConvertedTextForeground
                          IMESelectedConvertedTextUnderline SpellCheckerUnderline""".split()
    %>
    use gecko_bindings::bindings::Gecko_GetLookAndFeelSystemColor;
    use gecko_bindings::structs::root::mozilla::LookAndFeel_ColorID;
    use std::fmt::{self, Write};
    use style_traits::{CssWriter, ToCss};
    use values::computed::{Context, ToComputedValue};

    pub type SystemColor = LookAndFeel_ColorID;

    // It's hard to implement MallocSizeOf for LookAndFeel_ColorID because it
    // is a bindgen type. So we implement it on the typedef instead.
    malloc_size_of_is_0!(SystemColor);

    impl ToCss for SystemColor {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
        where
            W: Write,
        {
            let s = match *self {
                % for color in system_colors + extra_colors:
                    LookAndFeel_ColorID::eColorID_${to_rust_ident(color)} => "${color}",
                % endfor
                LookAndFeel_ColorID::eColorID_LAST_COLOR => unreachable!(),
            };
            dest.write_str(s)
        }
    }

    impl ToComputedValue for SystemColor {
        type ComputedValue = u32; // nscolor

        #[inline]
        fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
            unsafe {
                Gecko_GetLookAndFeelSystemColor(*self as i32,
                                                cx.device().pres_context())
            }
        }

        #[inline]
        fn from_computed_value(_: &Self::ComputedValue) -> Self {
            unreachable!()
        }
    }

    impl SystemColor {
        pub fn from_ident<'i, 't>(ident: &str) -> Result<Self, ()> {
            ascii_case_insensitive_phf_map! {
                color_name -> SystemColor = {
                    % for color in system_colors:
                        "${color}" => LookAndFeel_ColorID::eColorID_${to_rust_ident(color)},
                    % endfor
                }
            }

            color_name(ident).cloned().ok_or(())
        }
    }
}
% endif
