/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Color", inherited=True) %>

<% from data import to_rust_ident %>

<%helpers:longhand name="color" need_clone="True"
                   animation_value_type="IntermediateRGBA"
                   ignored_when_colors_disabled="True"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER"
                   spec="https://drafts.csswg.org/css-color/#color">
    use cssparser::RGBA;
    use values::specified::{AllowQuirks, Color};

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            self.0.to_computed_value(context)
                .to_rgba(context.builder.get_parent_color().clone_color())
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(Color::rgba(*computed).into())
        }
    }

    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[derive(Clone, Debug, PartialEq, ToCss)]
    pub struct SpecifiedValue(pub Color);
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use cssparser;
        pub type T = cssparser::RGBA;
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        RGBA::new(0, 0, 0, 255) // black
    }
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        Color::parse_quirky(context, input, AllowQuirks::Yes).map(SpecifiedValue)
    }

    // FIXME(#15973): Add servo support for system colors
    % if product == "gecko":
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
        use std::fmt;
        use style_traits::ToCss;

        pub type SystemColor = LookAndFeel_ColorID;

        impl ToCss for SystemColor {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
            pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
                ascii_case_insensitive_phf_map! {
                    color_name -> SystemColor = {
                        % for color in system_colors:
                            "${color}" => LookAndFeel_ColorID::eColorID_${to_rust_ident(color)},
                        % endfor
                    }
                }

                let ident = input.expect_ident()?;
                if let Some(color) = color_name(&ident) {
                    Ok(*color)
                } else {
                    Err(SelectorParseError::UnexpectedIdent(ident.clone()).into())
                }
            }
        }
    % endif
</%helpers:longhand>
