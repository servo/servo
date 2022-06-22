/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified color values.

use super::AllowQuirks;
use crate::parser::{Parse, ParserContext};
use crate::values::computed::{Color as ComputedColor, Context, ToComputedValue};
use crate::values::generics::color::{ColorInterpolationMethod, GenericColorMix, GenericCaretColor, GenericColorOrAuto};
use crate::values::specified::calc::CalcNode;
use crate::values::specified::Percentage;
use crate::values::CustomIdent;
use cssparser::{AngleOrNumber, Color as CSSParserColor, Parser, Token, RGBA};
use cssparser::{BasicParseErrorKind, NumberOrPercentage, ParseErrorKind};
use itoa;
use std::fmt::{self, Write};
use std::io::Write as IoWrite;
use style_traits::{CssType, CssWriter, KeywordsCollectFn, ParseError, StyleParseErrorKind};
use style_traits::{SpecifiedValueInfo, ToCss, ValueParseErrorKind};

/// A specified color-mix().
pub type ColorMix = GenericColorMix<Color, Percentage>;

#[inline]
fn allow_color_mix() -> bool {
    #[cfg(feature = "gecko")]
    return static_prefs::pref!("layout.css.color-mix.enabled");
    #[cfg(feature = "servo")]
    return false;
}

impl Parse for ColorMix {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let enabled =
            context.chrome_rules_enabled() || allow_color_mix();

        if !enabled {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        input.expect_function_matching("color-mix")?;

        input.parse_nested_block(|input| {
            let interpolation = ColorInterpolationMethod::parse(context, input)?;
            input.expect_comma()?;

            let try_parse_percentage = |input: &mut Parser| -> Option<Percentage> {
                input
                    .try_parse(|input| Percentage::parse_zero_to_a_hundred(context, input))
                    .ok()
            };

            let mut left_percentage = try_parse_percentage(input);

            let left = Color::parse(context, input)?;
            if left_percentage.is_none() {
                left_percentage = try_parse_percentage(input);
            }

            input.expect_comma()?;

            let mut right_percentage = try_parse_percentage(input);

            let right = Color::parse(context, input)?;

            if right_percentage.is_none() {
                right_percentage = try_parse_percentage(input);
            }

            let right_percentage = right_percentage
                .unwrap_or_else(|| Percentage::new(1.0 - left_percentage.map_or(0.5, |p| p.get())));

            let left_percentage =
                left_percentage.unwrap_or_else(|| Percentage::new(1.0 - right_percentage.get()));

            if left_percentage.get() + right_percentage.get() <= 0.0 {
                // If the percentages sum to zero, the function is invalid.
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }

            Ok(ColorMix {
                interpolation,
                left,
                left_percentage,
                right,
                right_percentage,
                normalize_weights: true,
            })
        })
    }
}

/// Specified color value
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub enum Color {
    /// The 'currentColor' keyword
    CurrentColor,
    /// A specific RGBA color
    Numeric {
        /// Parsed RGBA color
        parsed: RGBA,
        /// Authored representation
        authored: Option<Box<str>>,
    },
    /// A system color.
    #[cfg(feature = "gecko")]
    System(SystemColor),
    /// A color mix.
    ColorMix(Box<ColorMix>),
    /// Quirksmode-only rule for inheriting color from the body
    #[cfg(feature = "gecko")]
    InheritFromBodyQuirk,
}

/// System colors. A bunch of these are ad-hoc, others come from Windows:
///
///   https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getsyscolor
///
/// Others are HTML/CSS specific. Spec is:
///
///   https://drafts.csswg.org/css-color/#css-system-colors
///   https://drafts.csswg.org/css-color/#deprecated-system-colors
#[allow(missing_docs)]
#[cfg(feature = "gecko")]
#[derive(Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, ToCss, ToShmem)]
#[repr(u8)]
pub enum SystemColor {
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    TextSelectDisabledBackground,
    #[css(skip)]
    TextSelectAttentionBackground,
    #[css(skip)]
    TextSelectAttentionForeground,
    #[css(skip)]
    TextHighlightBackground,
    #[css(skip)]
    TextHighlightForeground,
    #[css(skip)]
    IMERawInputBackground,
    #[css(skip)]
    IMERawInputForeground,
    #[css(skip)]
    IMERawInputUnderline,
    #[css(skip)]
    IMESelectedRawTextBackground,
    #[css(skip)]
    IMESelectedRawTextForeground,
    #[css(skip)]
    IMESelectedRawTextUnderline,
    #[css(skip)]
    IMEConvertedTextBackground,
    #[css(skip)]
    IMEConvertedTextForeground,
    #[css(skip)]
    IMEConvertedTextUnderline,
    #[css(skip)]
    IMESelectedConvertedTextBackground,
    #[css(skip)]
    IMESelectedConvertedTextForeground,
    #[css(skip)]
    IMESelectedConvertedTextUnderline,
    #[css(skip)]
    SpellCheckerUnderline,
    #[css(skip)]
    ThemedScrollbar,
    #[css(skip)]
    ThemedScrollbarInactive,
    #[css(skip)]
    ThemedScrollbarThumb,
    #[css(skip)]
    ThemedScrollbarThumbHover,
    #[css(skip)]
    ThemedScrollbarThumbActive,
    #[css(skip)]
    ThemedScrollbarThumbInactive,
    Activeborder,
    /// Background in the (active) titlebar.
    Activecaption,
    Appworkspace,
    Background,
    Buttonface,
    Buttonhighlight,
    Buttonshadow,
    Buttontext,
    /// Text color in the (active) titlebar.
    Captiontext,
    #[parse(aliases = "-moz-field")]
    Field,
    /// Used for disabled field backgrounds.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozDisabledfield,
    #[parse(aliases = "-moz-fieldtext")]
    Fieldtext,

    Graytext,
    Highlight,
    Highlighttext,
    Inactiveborder,
    /// Background in the (inactive) titlebar.
    Inactivecaption,
    /// Text color in the (inactive) titlebar.
    Inactivecaptiontext,
    Infobackground,
    Infotext,
    Menu,
    Menutext,
    Scrollbar,
    Threeddarkshadow,
    Threedface,
    Threedhighlight,
    Threedlightshadow,
    Threedshadow,
    Window,
    Windowframe,
    Windowtext,
    MozButtondefault,
    #[parse(aliases = "-moz-default-color")]
    Canvastext,
    #[parse(aliases = "-moz-default-background-color")]
    Canvas,
    MozDialog,
    MozDialogtext,
    /// Used to highlight valid regions to drop something onto.
    MozDragtargetzone,
    /// Used for selected but not focused cell backgrounds.
    #[parse(aliases = "-moz-html-cellhighlight")]
    MozCellhighlight,
    /// Used for selected but not focused cell text.
    #[parse(aliases = "-moz-html-cellhighlighttext")]
    MozCellhighlighttext,
    /// Used for selected and focused html cell backgrounds.
    Selecteditem,
    /// Used for selected and focused html cell text.
    Selecteditemtext,
    /// Used to button text background when hovered.
    MozButtonhoverface,
    /// Used to button text color when hovered.
    MozButtonhovertext,
    /// Used for menu item backgrounds when hovered.
    MozMenuhover,
    /// Used for menu item text when hovered.
    MozMenuhovertext,
    /// Used for menubar item text.
    MozMenubartext,
    /// Used for menubar item text when hovered.
    MozMenubarhovertext,

    /// On platforms where these colors are the same as -moz-field, use
    /// -moz-fieldtext as foreground color
    MozEventreerow,
    MozOddtreerow,

    /// Used for button text when pressed.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozButtonactivetext,

    /// Used for button background when pressed.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozButtonactiveface,

    /// Used for button background when disabled.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozButtondisabledface,

    /// Background color of chrome toolbars in active windows.
    MozMacChromeActive,
    /// Background color of chrome toolbars in inactive windows.
    MozMacChromeInactive,
    /// Foreground color of default buttons.
    MozMacDefaultbuttontext,
    /// Ring color around text fields and lists.
    MozMacFocusring,
    /// Color used when mouse is over a menu item.
    MozMacMenuselect,
    /// Color used to do shadows on menu items.
    MozMacMenushadow,
    /// Color used to display text for disabled menu items.
    MozMacMenutextdisable,
    /// Color used to display text while mouse is over a menu item.
    MozMacMenutextselect,
    /// Text color of disabled text on toolbars.
    MozMacDisabledtoolbartext,
    /// Inactive light hightlight
    MozMacSecondaryhighlight,

    MozMacMenupopup,
    MozMacMenuitem,
    MozMacActiveMenuitem,
    MozMacSourceList,
    MozMacSourceListSelection,
    MozMacActiveSourceListSelection,
    MozMacTooltip,

    /// Theme accent color.
    /// https://drafts.csswg.org/css-color-4/#valdef-system-color-accentcolor
    Accentcolor,

    /// Foreground for the accent color.
    /// https://drafts.csswg.org/css-color-4/#valdef-system-color-accentcolortext
    Accentcolortext,

    /// The background-color for :autofill-ed inputs.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozAutofillBackground,

    /// Media rebar text.
    MozWinMediatext,
    /// Communications rebar text.
    MozWinCommunicationstext,

    /// Hyperlink color extracted from the system, not affected by the
    /// browser.anchor_color user pref.
    ///
    /// There is no OS-specified safe background color for this text, but it is
    /// used regularly within Windows and the Gnome DE on Dialog and Window
    /// colors.
    MozNativehyperlinktext,

    /// As above, but visited link color.
    #[css(skip)]
    MozNativevisitedhyperlinktext,

    #[parse(aliases = "-moz-hyperlinktext")]
    Linktext,
    #[parse(aliases = "-moz-activehyperlinktext")]
    Activetext,
    #[parse(aliases = "-moz-visitedhyperlinktext")]
    Visitedtext,

    /// Combobox widgets
    MozComboboxtext,
    MozCombobox,

    /// Color of tree column headers
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozColheadertext,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozColheaderhovertext,

    #[css(skip)]
    End, // Just for array-indexing purposes.
}

#[cfg(feature = "gecko")]
impl SystemColor {
    #[inline]
    fn compute(&self, cx: &Context) -> ComputedColor {
        use crate::gecko::values::convert_nscolor_to_rgba;
        use crate::gecko_bindings::bindings;

        // TODO: We should avoid cloning here most likely, though it's
        // cheap-ish.
        let style_color_scheme = cx.style().get_inherited_ui().clone_color_scheme();
        let color = cx.device().system_nscolor(*self, &style_color_scheme);
        if color == bindings::NS_SAME_AS_FOREGROUND_COLOR {
            return ComputedColor::currentcolor();
        }
        ComputedColor::rgba(convert_nscolor_to_rgba(color))
    }
}

impl From<RGBA> for Color {
    fn from(value: RGBA) -> Self {
        Color::rgba(value)
    }
}

struct ColorComponentParser<'a, 'b: 'a>(&'a ParserContext<'b>);
impl<'a, 'b: 'a, 'i: 'a> ::cssparser::ColorComponentParser<'i> for ColorComponentParser<'a, 'b> {
    type Error = StyleParseErrorKind<'i>;

    fn parse_angle_or_number<'t>(
        &self,
        input: &mut Parser<'i, 't>,
    ) -> Result<AngleOrNumber, ParseError<'i>> {
        use crate::values::specified::Angle;

        let location = input.current_source_location();
        let token = input.next()?.clone();
        match token {
            Token::Dimension {
                value, ref unit, ..
            } => {
                let angle = Angle::parse_dimension(value, unit, /* from_calc = */ false);

                let degrees = match angle {
                    Ok(angle) => angle.degrees(),
                    Err(()) => return Err(location.new_unexpected_token_error(token.clone())),
                };

                Ok(AngleOrNumber::Angle { degrees })
            },
            Token::Number { value, .. } => Ok(AngleOrNumber::Number { value }),
            Token::Function(ref name) => {
                let function = CalcNode::math_function(name, location)?;
                CalcNode::parse_angle_or_number(self.0, input, function)
            },
            t => return Err(location.new_unexpected_token_error(t)),
        }
    }

    fn parse_percentage<'t>(&self, input: &mut Parser<'i, 't>) -> Result<f32, ParseError<'i>> {
        Ok(Percentage::parse(self.0, input)?.get())
    }

    fn parse_number<'t>(&self, input: &mut Parser<'i, 't>) -> Result<f32, ParseError<'i>> {
        use crate::values::specified::Number;

        Ok(Number::parse(self.0, input)?.get())
    }

    fn parse_number_or_percentage<'t>(
        &self,
        input: &mut Parser<'i, 't>,
    ) -> Result<NumberOrPercentage, ParseError<'i>> {
        let location = input.current_source_location();

        match *input.next()? {
            Token::Number { value, .. } => Ok(NumberOrPercentage::Number { value }),
            Token::Percentage { unit_value, .. } => {
                Ok(NumberOrPercentage::Percentage { unit_value })
            },
            Token::Function(ref name) => {
                let function = CalcNode::math_function(name, location)?;
                CalcNode::parse_number_or_percentage(self.0, input, function)
            },
            ref t => return Err(location.new_unexpected_token_error(t.clone())),
        }
    }
}

impl Parse for Color {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // Currently we only store authored value for color keywords,
        // because all browsers serialize those values as keywords for
        // specified value.
        let start = input.state();
        let authored = input.expect_ident_cloned().ok();
        input.reset(&start);

        let compontent_parser = ColorComponentParser(&*context);
        match input.try_parse(|i| CSSParserColor::parse_with(&compontent_parser, i)) {
            Ok(value) => Ok(match value {
                CSSParserColor::CurrentColor => Color::CurrentColor,
                CSSParserColor::RGBA(rgba) => Color::Numeric {
                    parsed: rgba,
                    authored: authored.map(|s| s.to_ascii_lowercase().into_boxed_str()),
                },
            }),
            Err(e) => {
                #[cfg(feature = "gecko")]
                {
                    if let Ok(system) = input.try_parse(|i| SystemColor::parse(context, i)) {
                        return Ok(Color::System(system));
                    }
                }

                if let Ok(mix) = input.try_parse(|i| ColorMix::parse(context, i)) {
                    return Ok(Color::ColorMix(Box::new(mix)));
                }

                match e.kind {
                    ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken(t)) => {
                        Err(e.location.new_custom_error(StyleParseErrorKind::ValueError(
                            ValueParseErrorKind::InvalidColor(t),
                        )))
                    },
                    _ => Err(e),
                }
            },
        }
    }
}

impl ToCss for Color {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            Color::CurrentColor => CSSParserColor::CurrentColor.to_css(dest),
            Color::Numeric {
                authored: Some(ref authored),
                ..
            } => dest.write_str(authored),
            Color::Numeric {
                parsed: ref rgba, ..
            } => rgba.to_css(dest),
            Color::ColorMix(ref mix) => mix.to_css(dest),
            #[cfg(feature = "gecko")]
            Color::System(system) => system.to_css(dest),
            #[cfg(feature = "gecko")]
            Color::InheritFromBodyQuirk => Ok(()),
        }
    }
}

/// A wrapper of cssparser::Color::parse_hash.
///
/// That function should never return CurrentColor, so it makes no sense to
/// handle a cssparser::Color here. This should really be done in cssparser
/// directly rather than here.
fn parse_hash_color(value: &[u8]) -> Result<RGBA, ()> {
    CSSParserColor::parse_hash(value).map(|color| match color {
        CSSParserColor::RGBA(rgba) => rgba,
        CSSParserColor::CurrentColor => unreachable!("parse_hash should never return currentcolor"),
    })
}

impl Color {
    /// Returns whether this color is allowed in forced-colors mode.
    pub fn honored_in_forced_colors_mode(&self, allow_transparent: bool) -> bool {
        match *self {
            #[cfg(feature = "gecko")]
            Color::InheritFromBodyQuirk => false,
            Color::CurrentColor => false,
            #[cfg(feature = "gecko")]
            Color::System(..) => true,
            Color::Numeric { ref parsed, .. } => allow_transparent && parsed.alpha == 0,
            Color::ColorMix(ref mix) => {
                mix.left.honored_in_forced_colors_mode(allow_transparent) &&
                    mix.right.honored_in_forced_colors_mode(allow_transparent)
            },
        }
    }

    /// Returns currentcolor value.
    #[inline]
    pub fn currentcolor() -> Color {
        Color::CurrentColor
    }

    /// Returns transparent value.
    #[inline]
    pub fn transparent() -> Color {
        // We should probably set authored to "transparent", but maybe it doesn't matter.
        Color::rgba(RGBA::transparent())
    }

    /// Returns a numeric RGBA color value.
    #[inline]
    pub fn rgba(rgba: RGBA) -> Self {
        Color::Numeric {
            parsed: rgba,
            authored: None,
        }
    }

    /// Parse a color, with quirks.
    ///
    /// <https://quirks.spec.whatwg.org/#the-hashless-hex-color-quirk>
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        input.try_parse(|i| Self::parse(context, i)).or_else(|e| {
            if !allow_quirks.allowed(context.quirks_mode) {
                return Err(e);
            }
            Color::parse_quirky_color(input)
                .map(Color::rgba)
                .map_err(|_| e)
        })
    }

    /// Parse a <quirky-color> value.
    ///
    /// <https://quirks.spec.whatwg.org/#the-hashless-hex-color-quirk>
    fn parse_quirky_color<'i, 't>(input: &mut Parser<'i, 't>) -> Result<RGBA, ParseError<'i>> {
        let location = input.current_source_location();
        let (value, unit) = match *input.next()? {
            Token::Number {
                int_value: Some(integer),
                ..
            } => (integer, None),
            Token::Dimension {
                int_value: Some(integer),
                ref unit,
                ..
            } => (integer, Some(unit)),
            Token::Ident(ref ident) => {
                if ident.len() != 3 && ident.len() != 6 {
                    return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                return parse_hash_color(ident.as_bytes()).map_err(|()| {
                    location.new_custom_error(StyleParseErrorKind::UnspecifiedError)
                });
            },
            ref t => {
                return Err(location.new_unexpected_token_error(t.clone()));
            },
        };
        if value < 0 {
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        let length = if value <= 9 {
            1
        } else if value <= 99 {
            2
        } else if value <= 999 {
            3
        } else if value <= 9999 {
            4
        } else if value <= 99999 {
            5
        } else if value <= 999999 {
            6
        } else {
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        };
        let total = length + unit.as_ref().map_or(0, |d| d.len());
        if total > 6 {
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        let mut serialization = [b'0'; 6];
        let space_padding = 6 - total;
        let mut written = space_padding;
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        (&mut serialization[written..]).write_all(s.as_bytes()).unwrap();
        written += s.len();
        if let Some(unit) = unit {
            written += (&mut serialization[written..])
                .write(unit.as_bytes())
                .unwrap();
        }
        debug_assert_eq!(written, 6);
        parse_hash_color(&serialization)
            .map_err(|()| location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

impl Color {
    /// Converts this Color into a ComputedColor.
    ///
    /// If `context` is `None`, and the specified color requires data from
    /// the context to resolve, then `None` is returned.
    pub fn to_computed_color(&self, context: Option<&Context>) -> Option<ComputedColor> {
        Some(match *self {
            Color::CurrentColor => ComputedColor::CurrentColor,
            Color::Numeric { ref parsed, .. } => ComputedColor::Numeric(*parsed),
            Color::ColorMix(ref mix) => {
                use crate::values::computed::percentage::Percentage;

                let left = mix.left.to_computed_color(context)?;
                let right = mix.right.to_computed_color(context)?;
                let mut color = ComputedColor::ColorMix(Box::new(GenericColorMix {
                    interpolation: mix.interpolation,
                    left,
                    left_percentage: Percentage(mix.left_percentage.get()),
                    right,
                    right_percentage: Percentage(mix.right_percentage.get()),
                    normalize_weights: mix.normalize_weights,
                }));
                color.simplify(None);
                color
            },
            #[cfg(feature = "gecko")]
            Color::System(system) => system.compute(context?),
            #[cfg(feature = "gecko")]
            Color::InheritFromBodyQuirk => ComputedColor::rgba(context?.device().body_text_color()),
        })
    }
}

impl ToComputedValue for Color {
    type ComputedValue = ComputedColor;

    fn to_computed_value(&self, context: &Context) -> ComputedColor {
        self.to_computed_color(Some(context)).unwrap()
    }

    fn from_computed_value(computed: &ComputedColor) -> Self {
        match *computed {
            ComputedColor::Numeric(ref color) => Color::rgba(*color),
            ComputedColor::CurrentColor => Color::CurrentColor,
            ComputedColor::ColorMix(ref mix) => {
                Color::ColorMix(Box::new(ToComputedValue::from_computed_value(&**mix)))
            },
        }
    }
}

/// Specified color value for `-moz-font-smoothing-background-color`.
///
/// This property does not support `currentcolor`. We could drop it at
/// parse-time, but it's not exposed to the web so it doesn't really matter.
///
/// We resolve it to `transparent` instead.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct MozFontSmoothingBackgroundColor(pub Color);

impl Parse for MozFontSmoothingBackgroundColor {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Color::parse(context, input).map(MozFontSmoothingBackgroundColor)
    }
}

impl ToComputedValue for MozFontSmoothingBackgroundColor {
    type ComputedValue = RGBA;

    fn to_computed_value(&self, context: &Context) -> RGBA {
        self.0
            .to_computed_value(context)
            .into_rgba(RGBA::transparent())
    }

    fn from_computed_value(computed: &RGBA) -> Self {
        MozFontSmoothingBackgroundColor(Color::rgba(*computed))
    }
}

impl SpecifiedValueInfo for Color {
    const SUPPORTED_TYPES: u8 = CssType::COLOR;

    fn collect_completion_keywords(f: KeywordsCollectFn) {
        // We are not going to insert all the color names here. Caller and
        // devtools should take care of them. XXX Actually, transparent
        // should probably be handled that way as well.
        // XXX `currentColor` should really be `currentcolor`. But let's
        // keep it consistent with the old system for now.
        f(&[
            "rgb",
            "rgba",
            "hsl",
            "hsla",
            "hwb",
            "currentColor",
            "transparent",
        ]);
    }
}

/// Specified value for the "color" property, which resolves the `currentcolor`
/// keyword to the parent color instead of self's color.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct ColorPropertyValue(pub Color);

impl ToComputedValue for ColorPropertyValue {
    type ComputedValue = RGBA;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> RGBA {
        self.0
            .to_computed_value(context)
            .into_rgba(context.builder.get_parent_inherited_text().clone_color())
    }

    #[inline]
    fn from_computed_value(computed: &RGBA) -> Self {
        ColorPropertyValue(Color::rgba(*computed).into())
    }
}

impl Parse for ColorPropertyValue {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Color::parse_quirky(context, input, AllowQuirks::Yes).map(ColorPropertyValue)
    }
}

/// auto | <color>
pub type ColorOrAuto = GenericColorOrAuto<Color>;

/// caret-color
pub type CaretColor = GenericCaretColor<Color>;

impl Parse for CaretColor {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        ColorOrAuto::parse(context, input).map(GenericCaretColor)
    }
}

bitflags! {
    /// Various flags to represent the color-scheme property in an efficient
    /// way.
    #[derive(Default, MallocSizeOf, SpecifiedValueInfo, ToComputedValue, ToResolvedValue, ToShmem)]
    #[repr(C)]
    #[value_info(other_values = "light,dark,only")]
    pub struct ColorSchemeFlags: u8 {
        /// Whether the author specified `light`.
        const LIGHT = 1 << 0;
        /// Whether the author specified `dark`.
        const DARK = 1 << 1;
        /// Whether the author specified `only`.
        const ONLY = 1 << 2;
    }
}

/// <https://drafts.csswg.org/css-color-adjust/#color-scheme-prop>
#[derive(
    Clone,
    Debug,
    Default,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
#[value_info(other_values = "normal")]
pub struct ColorScheme {
    #[ignore_malloc_size_of = "Arc"]
    idents: crate::ArcSlice<CustomIdent>,
    bits: ColorSchemeFlags,
}

impl ColorScheme {
    /// Returns the `normal` value.
    pub fn normal() -> Self {
        Self {
            idents: Default::default(),
            bits: ColorSchemeFlags::empty(),
        }
    }

    /// Returns the raw bitfield.
    pub fn raw_bits(&self) -> u8 {
        self.bits.bits
    }
}

impl Parse for ColorScheme {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut idents = vec![];
        let mut bits = ColorSchemeFlags::empty();

        let mut location = input.current_source_location();
        while let Ok(ident) = input.try_parse(|i| i.expect_ident_cloned()) {
            let mut is_only = false;
            match_ignore_ascii_case! { &ident,
                "normal" => {
                    if idents.is_empty() && bits.is_empty() {
                        return Ok(Self::normal());
                    }
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                },
                "light" => bits.insert(ColorSchemeFlags::LIGHT),
                "dark" => bits.insert(ColorSchemeFlags::DARK),
                "only" => {
                    if bits.intersects(ColorSchemeFlags::ONLY) {
                        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                    }
                    bits.insert(ColorSchemeFlags::ONLY);
                    is_only = true;
                },
                _ => {},
            };

            if is_only {
                if !idents.is_empty() {
                    // Only is allowed either at the beginning or at the end,
                    // but not in the middle.
                    break;
                }
            } else {
                idents.push(CustomIdent::from_ident(location, &ident, &[])?);
            }
            location = input.current_source_location();
        }

        if idents.is_empty() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(Self {
            idents: crate::ArcSlice::from_iter(idents.into_iter()),
            bits,
        })
    }
}

impl ToCss for ColorScheme {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.idents.is_empty() {
            debug_assert!(self.bits.is_empty());
            return dest.write_str("normal");
        }
        let mut first = true;
        for ident in self.idents.iter() {
            if !first {
                dest.write_char(' ')?;
            }
            first = false;
            ident.to_css(dest)?;
        }
        if self.bits.intersects(ColorSchemeFlags::ONLY) {
            dest.write_str(" only")?;
        }
        Ok(())
    }
}

/// https://drafts.csswg.org/css-color-adjust/#print-color-adjust
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum PrintColorAdjust {
    /// Ignore backgrounds and darken text.
    Economy,
    /// Respect specified colors.
    Exact,
}
