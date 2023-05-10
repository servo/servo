/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified color values.

use super::AllowQuirks;
use crate::color::mix::ColorInterpolationMethod;
use crate::color::{AbsoluteColor, ColorComponents, ColorFlags, ColorSpace};
use crate::media_queries::Device;
use crate::parser::{Parse, ParserContext};
use crate::values::computed::{Color as ComputedColor, Context, ToComputedValue};
use crate::values::generics::color::{GenericCaretColor, GenericColorMix, GenericColorOrAuto};
use crate::values::specified::calc::CalcNode;
use crate::values::specified::Percentage;
use crate::values::CustomIdent;
use cssparser::{AngleOrNumber, Color as CSSParserColor, Parser, Token};
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
    return true;
}

#[inline]
fn allow_more_color_4() -> bool {
    #[cfg(feature = "gecko")]
    return static_prefs::pref!("layout.css.more_color_4.enabled");
    #[cfg(feature = "servo")]
    return true;
}

impl ColorMix {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        preserve_authored: PreserveAuthored,
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

            let left = Color::parse_internal(context, input, preserve_authored)?;
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

/// Container holding an absolute color and the text specified by an author.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub struct Absolute {
    /// The specified color.
    pub color: AbsoluteColor,
    /// Authored representation.
    pub authored: Option<Box<str>>,
}

impl ToCss for Absolute {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if let Some(ref authored) = self.authored {
            dest.write_str(authored)
        } else {
            self.color.to_css(dest)
        }
    }
}

/// Specified color value
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub enum Color {
    /// The 'currentColor' keyword
    CurrentColor,
    /// An absolute color.
    /// https://w3c.github.io/csswg-drafts/css-color-4/#typedef-absolute-color-function
    Absolute(Box<Absolute>),
    /// A system color.
    #[cfg(feature = "gecko")]
    System(SystemColor),
    /// A color mix.
    ColorMix(Box<ColorMix>),
    /// Quirksmode-only rule for inheriting color from the body
    #[cfg(feature = "gecko")]
    InheritFromBodyQuirk,
}

impl From<AbsoluteColor> for Color {
    #[inline]
    fn from(value: AbsoluteColor) -> Self {
        Self::from_absolute_color(value)
    }
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
    Activeborder,
    /// Background in the (active) titlebar.
    Activecaption,
    Appworkspace,
    Background,
    Buttonface,
    Buttonhighlight,
    Buttonshadow,
    Buttontext,
    Buttonborder,
    /// Text color in the (active) titlebar.
    Captiontext,
    #[parse(aliases = "-moz-field")]
    Field,
    /// Used for disabled field backgrounds.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozDisabledfield,
    #[parse(aliases = "-moz-fieldtext")]
    Fieldtext,

    Mark,
    Marktext,

    /// Combobox widgets
    MozComboboxtext,
    MozCombobox,

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
    /// Used for menu item backgrounds when hovered and disabled.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMenuhoverdisabled,
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

    /// Color of tree column headers
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozColheadertext,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozColheaderhovertext,

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

    #[css(skip)]
    End, // Just for array-indexing purposes.
}

#[cfg(feature = "gecko")]
impl SystemColor {
    #[inline]
    fn compute(&self, cx: &Context) -> ComputedColor {
        use crate::gecko::values::convert_nscolor_to_absolute_color;
        use crate::gecko_bindings::bindings;

        // TODO: We should avoid cloning here most likely, though it's
        // cheap-ish.
        let style_color_scheme = cx.style().get_inherited_ui().clone_color_scheme();
        let color = cx.device().system_nscolor(*self, &style_color_scheme);
        if color == bindings::NS_SAME_AS_FOREGROUND_COLOR {
            return ComputedColor::currentcolor();
        }
        ComputedColor::Absolute(convert_nscolor_to_absolute_color(color))
    }
}

#[inline]
fn new_absolute(
    color_space: ColorSpace,
    c1: Option<f32>,
    c2: Option<f32>,
    c3: Option<f32>,
    alpha: Option<f32>,
) -> Color {
    let mut flags = ColorFlags::empty();

    macro_rules! c {
        ($v:expr,$flag:tt) => {{
            if let Some(value) = $v {
                value
            } else {
                flags |= ColorFlags::$flag;
                0.0
            }
        }};
    }

    let c1 = c!(c1, C1_IS_NONE);
    let c2 = c!(c2, C2_IS_NONE);
    let c3 = c!(c3, C3_IS_NONE);
    let alpha = c!(alpha, ALPHA_IS_NONE);

    let mut color = AbsoluteColor::new(color_space, ColorComponents(c1, c2, c3), alpha);
    color.flags |= flags;
    Color::Absolute(Box::new(Absolute {
        color,
        authored: None,
    }))
}

impl cssparser::FromParsedColor for Color {
    fn from_current_color() -> Self {
        Color::CurrentColor
    }

    fn from_rgba(red: Option<u8>, green: Option<u8>, blue: Option<u8>, alpha: Option<f32>) -> Self {
        new_absolute(
            ColorSpace::Srgb,
            red.map(|r| r as f32 / 255.0),
            green.map(|g| g as f32 / 255.0),
            blue.map(|b| b as f32 / 255.0),
            alpha,
        )
    }

    fn from_hsl(
        hue: Option<f32>,
        saturation: Option<f32>,
        lightness: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        new_absolute(ColorSpace::Hsl, hue, saturation, lightness, alpha)
    }

    fn from_hwb(
        hue: Option<f32>,
        whiteness: Option<f32>,
        blackness: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        new_absolute(ColorSpace::Hwb, hue, whiteness, blackness, alpha)
    }

    fn from_lab(
        lightness: Option<f32>,
        a: Option<f32>,
        b: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        new_absolute(ColorSpace::Lab, lightness, a, b, alpha)
    }

    fn from_lch(
        lightness: Option<f32>,
        chroma: Option<f32>,
        hue: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        new_absolute(ColorSpace::Lch, lightness, chroma, hue, alpha)
    }

    fn from_oklab(
        lightness: Option<f32>,
        a: Option<f32>,
        b: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        new_absolute(ColorSpace::Oklab, lightness, a, b, alpha)
    }

    fn from_oklch(
        lightness: Option<f32>,
        chroma: Option<f32>,
        hue: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        new_absolute(ColorSpace::Oklch, lightness, chroma, hue, alpha)
    }

    fn from_color_function(
        color_space: cssparser::PredefinedColorSpace,
        c1: Option<f32>,
        c2: Option<f32>,
        c3: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        let mut result = new_absolute(color_space.into(), c1, c2, c3, alpha);
        if let Color::Absolute(ref mut absolute) = result {
            if matches!(absolute.color.color_space, ColorSpace::Srgb) {
                absolute.color.flags |= ColorFlags::AS_COLOR_FUNCTION;
            }
        }
        result
    }
}

struct ColorParser<'a, 'b: 'a>(&'a ParserContext<'b>);
impl<'a, 'b: 'a, 'i: 'a> ::cssparser::ColorParser<'i> for ColorParser<'a, 'b> {
    type Output = Color;
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
                let function = CalcNode::math_function(self.0, name, location)?;
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
                let function = CalcNode::math_function(self.0, name, location)?;
                CalcNode::parse_number_or_percentage(self.0, input, function)
            },
            ref t => return Err(location.new_unexpected_token_error(t.clone())),
        }
    }
}

/// Whether to preserve authored colors during parsing. That's useful only if we
/// plan to serialize the color back.
enum PreserveAuthored {
    No,
    Yes,
}

impl Parse for Color {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, PreserveAuthored::Yes)
    }
}

impl Color {
    fn parse_internal<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        preserve_authored: PreserveAuthored,
    ) -> Result<Self, ParseError<'i>> {
        let authored = match preserve_authored {
            PreserveAuthored::No => None,
            PreserveAuthored::Yes => {
                // Currently we only store authored value for color keywords,
                // because all browsers serialize those values as keywords for
                // specified value.
                let start = input.state();
                let authored = input.expect_ident_cloned().ok();
                input.reset(&start);
                authored
            },
        };

        let color_parser = ColorParser(&*context);
        match input.try_parse(|i| cssparser::parse_color_with(&color_parser, i)) {
            Ok(mut color) => {
                if let Color::Absolute(ref mut absolute) = color {
                    let enabled = {
                        let is_legacy_color = matches!(
                            absolute.color.color_space,
                            ColorSpace::Srgb | ColorSpace::Hsl
                        );
                        let is_color_function =
                            absolute.color.flags.contains(ColorFlags::AS_COLOR_FUNCTION);
                        let pref_enabled = allow_more_color_4();

                        (is_legacy_color && !is_color_function) || pref_enabled
                    };
                    if !enabled {
                        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                    }

                    // Because we can't set the `authored` value at construction time, we have to set it
                    // here.
                    absolute.authored = authored.map(|s| s.to_ascii_lowercase().into_boxed_str());
                }
                Ok(color)
            },
            Err(e) => {
                #[cfg(feature = "gecko")]
                {
                    if let Ok(system) = input.try_parse(|i| SystemColor::parse(context, i)) {
                        return Ok(Color::System(system));
                    }
                }

                if let Ok(mix) = input.try_parse(|i| ColorMix::parse(context, i, preserve_authored))
                {
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

    /// Returns whether a given color is valid for authors.
    pub fn is_valid(context: &ParserContext, input: &mut Parser) -> bool {
        input
            .parse_entirely(|input| Self::parse_internal(context, input, PreserveAuthored::No))
            .is_ok()
    }

    /// Tries to parse a color and compute it with a given device.
    pub fn parse_and_compute(
        context: &ParserContext,
        input: &mut Parser,
        device: Option<&Device>,
    ) -> Option<ComputedColor> {
        use crate::error_reporting::ContextualParseError;
        let start = input.position();
        let result = input
            .parse_entirely(|input| Self::parse_internal(context, input, PreserveAuthored::No));

        let specified = match result {
            Ok(s) => s,
            Err(e) => {
                if !context.error_reporting_enabled() {
                    return None;
                }
                // Ignore other kinds of errors that might be reported, such as
                // ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken),
                // since Gecko didn't use to report those to the error console.
                //
                // TODO(emilio): Revise whether we want to keep this at all, we
                // use this only for canvas, this warnings are disabled by
                // default and not available on OffscreenCanvas anyways...
                if let ParseErrorKind::Custom(StyleParseErrorKind::ValueError(..)) = e.kind {
                    let location = e.location.clone();
                    let error = ContextualParseError::UnsupportedValue(input.slice_from(start), e);
                    context.log_css_error(location, error);
                }
                return None;
            },
        };

        match device {
            Some(device) => {
                Context::for_media_query_evaluation(device, device.quirks_mode(), |context| {
                    specified.to_computed_color(Some(&context))
                })
            },
            None => specified.to_computed_color(None),
        }
    }
}

impl ToCss for Color {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            Color::CurrentColor => cssparser::ToCss::to_css(&CSSParserColor::CurrentColor, dest),
            Color::Absolute(ref absolute) => absolute.to_css(dest),
            Color::ColorMix(ref mix) => mix.to_css(dest),
            #[cfg(feature = "gecko")]
            Color::System(system) => system.to_css(dest),
            #[cfg(feature = "gecko")]
            Color::InheritFromBodyQuirk => Ok(()),
        }
    }
}

impl Color {
    /// Returns whether this color is allowed in forced-colors mode.
    pub fn honored_in_forced_colors_mode(&self, allow_transparent: bool) -> bool {
        match *self {
            #[cfg(feature = "gecko")]
            Self::InheritFromBodyQuirk => false,
            Self::CurrentColor => true,
            #[cfg(feature = "gecko")]
            Self::System(..) => true,
            Self::Absolute(ref absolute) => allow_transparent && absolute.color.alpha() == 0.0,
            Self::ColorMix(ref mix) => {
                mix.left.honored_in_forced_colors_mode(allow_transparent) &&
                    mix.right.honored_in_forced_colors_mode(allow_transparent)
            },
        }
    }

    /// Returns currentcolor value.
    #[inline]
    pub fn currentcolor() -> Self {
        Self::CurrentColor
    }

    /// Returns transparent value.
    #[inline]
    pub fn transparent() -> Self {
        // We should probably set authored to "transparent", but maybe it doesn't matter.
        Self::from_absolute_color(AbsoluteColor::transparent())
    }

    /// Create a color from an [`AbsoluteColor`].
    pub fn from_absolute_color(color: AbsoluteColor) -> Self {
        Color::Absolute(Box::new(Absolute {
            color,
            authored: None,
        }))
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
            Color::parse_quirky_color(input).map_err(|_| e)
        })
    }

    /// Parse a <quirky-color> value.
    ///
    /// <https://quirks.spec.whatwg.org/#the-hashless-hex-color-quirk>
    fn parse_quirky_color<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
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
                return cssparser::parse_hash_color(ident.as_bytes()).map_err(|()| {
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
        (&mut serialization[written..])
            .write_all(s.as_bytes())
            .unwrap();
        written += s.len();
        if let Some(unit) = unit {
            written += (&mut serialization[written..])
                .write(unit.as_bytes())
                .unwrap();
        }
        debug_assert_eq!(written, 6);
        cssparser::parse_hash_color(&serialization)
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
            Color::Absolute(ref absolute) => ComputedColor::Absolute(absolute.color),
            Color::ColorMix(ref mix) => {
                use crate::values::computed::percentage::Percentage;

                let left = mix.left.to_computed_color(context)?;
                let right = mix.right.to_computed_color(context)?;

                ComputedColor::from_color_mix(GenericColorMix {
                    interpolation: mix.interpolation,
                    left,
                    left_percentage: Percentage(mix.left_percentage.get()),
                    right,
                    right_percentage: Percentage(mix.right_percentage.get()),
                    normalize_weights: mix.normalize_weights,
                })
            },
            #[cfg(feature = "gecko")]
            Color::System(system) => system.compute(context?),
            #[cfg(feature = "gecko")]
            Color::InheritFromBodyQuirk => {
                ComputedColor::Absolute(context?.device().body_text_color())
            },
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
            ComputedColor::Absolute(ref color) => Self::from_absolute_color(color.clone()),
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
    type ComputedValue = AbsoluteColor;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        self.0
            .to_computed_value(context)
            .resolve_to_absolute(&AbsoluteColor::transparent())
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        MozFontSmoothingBackgroundColor(Color::from_absolute_color(*computed))
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
        if allow_color_mix() {
            f(&["color-mix"]);
        }
        if allow_more_color_4() {
            f(&["color", "lab", "lch", "oklab", "oklch"]);
        }
    }
}

/// Specified value for the "color" property, which resolves the `currentcolor`
/// keyword to the parent color instead of self's color.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct ColorPropertyValue(pub Color);

impl ToComputedValue for ColorPropertyValue {
    type ComputedValue = AbsoluteColor;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        let current_color = context.builder.get_parent_inherited_text().clone_color();
        self.0
            .to_computed_value(context)
            .resolve_to_absolute(&current_color)
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        ColorPropertyValue(Color::from_absolute_color(*computed).into())
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

/// https://drafts.csswg.org/css-color-adjust-1/#forced-color-adjust-prop
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
pub enum ForcedColorAdjust {
    /// Adjust colors if needed.
    Auto,
    /// Respect specified colors.
    None,
}
