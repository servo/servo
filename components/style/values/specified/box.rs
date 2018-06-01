/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for box properties.

use Atom;
use cssparser::Parser;
use custom_properties::Name as CustomPropertyName;
use parser::{Parse, ParserContext};
use properties::{LonghandId, ShorthandId, PropertyId, PropertyFlags, PropertyDeclarationId};
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use style_traits::{CssWriter, KeywordsCollectFn, ParseError, StyleParseErrorKind, SpecifiedValueInfo, ToCss};
use values::{CustomIdent, KeyframesName};
use values::generics::box_::AnimationIterationCount as GenericAnimationIterationCount;
use values::generics::box_::Perspective as GenericPerspective;
use values::generics::box_::VerticalAlign as GenericVerticalAlign;
use values::specified::{AllowQuirks, Number};
use values::specified::length::{LengthOrPercentage, NonNegativeLength};

#[cfg(feature = "gecko")]
fn moz_display_values_enabled(context: &ParserContext) -> bool {
    use gecko_bindings::structs;
    use stylesheets::Origin;
    context.stylesheet_origin == Origin::UserAgent ||
    context.chrome_rules_enabled() ||
    unsafe {
        structs::StaticPrefs_sVarCache_layout_css_xul_display_values_content_enabled
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToComputedValue, ToCss)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
/// Defines an element’s display type, which consists of
/// the two basic qualities of how an element generates boxes
/// <https://drafts.csswg.org/css-display/#propdef-display>
pub enum Display {
    Inline,
    Block,
    InlineBlock,
    Table,
    InlineTable,
    TableRowGroup,
    TableHeaderGroup,
    TableFooterGroup,
    TableRow,
    TableColumnGroup,
    TableColumn,
    TableCell,
    TableCaption,
    ListItem,
    None,
    #[css(aliases = "-webkit-flex")]
    Flex,
    #[css(aliases = "-webkit-inline-flex")]
    InlineFlex,
    #[cfg(feature = "gecko")]
    Grid,
    #[cfg(feature = "gecko")]
    InlineGrid,
    #[cfg(feature = "gecko")]
    Ruby,
    #[cfg(feature = "gecko")]
    RubyBase,
    #[cfg(feature = "gecko")]
    RubyBaseContainer,
    #[cfg(feature = "gecko")]
    RubyText,
    #[cfg(feature = "gecko")]
    RubyTextContainer,
    #[cfg(feature = "gecko")]
    Contents,
    #[cfg(feature = "gecko")]
    FlowRoot,
    #[cfg(feature = "gecko")]
    WebkitBox,
    #[cfg(feature = "gecko")]
    WebkitInlineBox,
    #[cfg(feature = "gecko")]
    MozBox,
    #[cfg(feature = "gecko")]
    MozInlineBox,
    #[cfg(feature = "gecko")]
    #[css(parse_condition = "moz_display_values_enabled")]
    MozGrid,
    #[cfg(feature = "gecko")]
    #[css(parse_condition = "moz_display_values_enabled")]
    MozInlineGrid,
    #[cfg(feature = "gecko")]
    #[css(parse_condition = "moz_display_values_enabled")]
    MozGridGroup,
    #[cfg(feature = "gecko")]
    #[css(parse_condition = "moz_display_values_enabled")]
    MozGridLine,
    #[cfg(feature = "gecko")]
    #[css(parse_condition = "moz_display_values_enabled")]
    MozStack,
    #[cfg(feature = "gecko")]
    #[css(parse_condition = "moz_display_values_enabled")]
    MozInlineStack,
    #[cfg(feature = "gecko")]
    #[css(parse_condition = "moz_display_values_enabled")]
    MozDeck,
    #[cfg(feature = "gecko")]
    #[css(parse_condition = "moz_display_values_enabled")]
    MozPopup,
    #[cfg(feature = "gecko")]
    #[css(parse_condition = "moz_display_values_enabled")]
    MozGroupbox,
}

impl Display {
    /// The initial display value.
    #[inline]
    pub fn inline() -> Self {
        Display::Inline
    }

    /// Returns whether this "display" value is the display of a flex or
    /// grid container.
    ///
    /// This is used to implement various style fixups.
    pub fn is_item_container(&self) -> bool {
        match *self {
            Display::Flex | Display::InlineFlex => true,
            #[cfg(feature = "gecko")]
            Display::Grid | Display::InlineGrid => true,
            _ => false,
        }
    }

    /// Returns whether an element with this display type is a line
    /// participant, which means it may lay its children on the same
    /// line as itself.
    pub fn is_line_participant(&self) -> bool {
        match *self {
            Display::Inline => true,
            #[cfg(feature = "gecko")]
            Display::Contents | Display::Ruby | Display::RubyBaseContainer => true,
            _ => false,
        }
    }

    /// Whether `new_display` should be ignored, given a previous
    /// `old_display` value.
    ///
    /// This is used to ignore `display: -moz-box` declarations after an
    /// equivalent `display: -webkit-box` declaration, since the former
    /// has a vastly different meaning. See bug 1107378 and bug 1407701.
    ///
    /// FIXME(emilio): This is a pretty decent hack, we should try to
    /// remove it.
    pub fn should_ignore_parsed_value(_old_display: Self, _new_display: Self) -> bool {
        #[cfg(feature = "gecko")]
        {
            match (_old_display, _new_display) {
                (Display::WebkitBox, Display::MozBox) |
                (Display::WebkitInlineBox, Display::MozInlineBox) => {
                    return true;
                },
                _ => {},
            }
        }

        return false;
    }

    /// Returns whether this "display" value is one of the types for
    /// ruby.
    #[cfg(feature = "gecko")]
    pub fn is_ruby_type(&self) -> bool {
        matches!(
            *self,
            Display::Ruby | Display::RubyBase | Display::RubyText | Display::RubyBaseContainer |
                Display::RubyTextContainer
        )
    }

    /// Returns whether this "display" value is a ruby level container.
    #[cfg(feature = "gecko")]
    pub fn is_ruby_level_container(&self) -> bool {
        matches!(
            *self,
            Display::RubyBaseContainer | Display::RubyTextContainer
        )
    }

    /// Convert this display into an equivalent block display.
    ///
    /// Also used for style adjustments.
    pub fn equivalent_block_display(&self, _is_root_element: bool) -> Self {
        match *self {
            // Values that have a corresponding block-outside version.
            Display::InlineTable => Display::Table,
            Display::InlineFlex => Display::Flex,

            #[cfg(feature = "gecko")]
            Display::InlineGrid => Display::Grid,
            #[cfg(feature = "gecko")]
            Display::WebkitInlineBox => Display::WebkitBox,

            // Special handling for contents and list-item on the root
            // element for Gecko.
            #[cfg(feature = "gecko")]
            Display::Contents | Display::ListItem if _is_root_element =>
            {
                Display::Block
            },

            // These are not changed by blockification.
            Display::None | Display::Block | Display::Flex | Display::ListItem | Display::Table => {
                *self
            },

            #[cfg(feature = "gecko")]
            Display::Contents | Display::FlowRoot | Display::Grid | Display::WebkitBox => *self,

            // Everything else becomes block.
            _ => Display::Block,
        }
    }

    /// Convert this display into an inline-outside display.
    ///
    /// Ideally it should implement spec: https://drafts.csswg.org/css-display/#inlinify
    /// but the spec isn't stable enough, so we copy what Gecko does for now.
    #[cfg(feature = "gecko")]
    pub fn inlinify(&self) -> Self {
        match *self {
            Display::Block | Display::FlowRoot => Display::InlineBlock,
            Display::Table => Display::InlineTable,
            Display::Flex => Display::InlineFlex,
            Display::Grid => Display::InlineGrid,
            // XXX bug 1105868 this should probably be InlineListItem:
            Display::ListItem => Display::Inline,
            Display::MozBox => Display::MozInlineBox,
            Display::MozStack => Display::MozInlineStack,
            Display::WebkitBox => Display::WebkitInlineBox,
            other => other,
        }
    }

    /// Returns true if the value is `Contents`
    #[inline]
    pub fn is_contents(&self) -> bool {
        match *self {
            #[cfg(feature = "gecko")]
            Display::Contents => true,
            _ => false,
        }
    }

    /// Returns true if the value is `None`
    #[inline]
    pub fn is_none(&self) -> bool {
        *self == Display::None
    }
}

/// A specified value for the `vertical-align` property.
pub type VerticalAlign = GenericVerticalAlign<LengthOrPercentage>;

impl Parse for VerticalAlign {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(lop) =
            input.try(|i| LengthOrPercentage::parse_quirky(context, i, AllowQuirks::Yes))
        {
            return Ok(GenericVerticalAlign::Length(lop));
        }

        try_match_ident_ignore_ascii_case! { input,
            "baseline" => Ok(GenericVerticalAlign::Baseline),
            "sub" => Ok(GenericVerticalAlign::Sub),
            "super" => Ok(GenericVerticalAlign::Super),
            "top" => Ok(GenericVerticalAlign::Top),
            "text-top" => Ok(GenericVerticalAlign::TextTop),
            "middle" => Ok(GenericVerticalAlign::Middle),
            "bottom" => Ok(GenericVerticalAlign::Bottom),
            "text-bottom" => Ok(GenericVerticalAlign::TextBottom),
            #[cfg(feature = "gecko")]
            "-moz-middle-with-baseline" => {
                Ok(GenericVerticalAlign::MozMiddleWithBaseline)
            },
        }
    }
}

/// https://drafts.csswg.org/css-animations/#animation-iteration-count
pub type AnimationIterationCount = GenericAnimationIterationCount<Number>;

impl Parse for AnimationIterationCount {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut ::cssparser::Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input
            .try(|input| input.expect_ident_matching("infinite"))
            .is_ok()
        {
            return Ok(GenericAnimationIterationCount::Infinite);
        }

        let number = Number::parse_non_negative(context, input)?;
        Ok(GenericAnimationIterationCount::Number(number))
    }
}

impl AnimationIterationCount {
    /// Returns the value `1.0`.
    #[inline]
    pub fn one() -> Self {
        GenericAnimationIterationCount::Number(Number::new(1.0))
    }
}

/// A value for the `animation-name` property.
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue)]
#[value_info(other_values = "none")]
pub struct AnimationName(pub Option<KeyframesName>);

impl AnimationName {
    /// Get the name of the animation as an `Atom`.
    pub fn as_atom(&self) -> Option<&Atom> {
        self.0.as_ref().map(|n| n.as_atom())
    }

    /// Returns the `none` value.
    pub fn none() -> Self {
        AnimationName(None)
    }
}

impl ToCss for AnimationName {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match self.0 {
            Some(ref name) => name.to_css(dest),
            None => dest.write_str("none"),
        }
    }
}

impl Parse for AnimationName {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(name) = input.try(|input| KeyframesName::parse(context, input)) {
            return Ok(AnimationName(Some(name)));
        }

        input.expect_ident_matching("none")?;
        Ok(AnimationName(None))
    }
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToComputedValue, ToCss)]
pub enum ScrollSnapType {
    None,
    Mandatory,
    Proximity,
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToComputedValue, ToCss)]
pub enum OverscrollBehavior {
    Auto,
    Contain,
    None,
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToComputedValue, ToCss)]
pub enum OverflowClipBox {
    PaddingBox,
    ContentBox,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
/// Provides a rendering hint to the user agent,
/// stating what kinds of changes the author expects
/// to perform on the element
///
/// <https://drafts.csswg.org/css-will-change/#will-change>
pub enum WillChange {
    /// Expresses no particular intent
    Auto,
    /// <custom-ident>
    #[css(comma)]
    AnimateableFeatures {
        /// The features that are supposed to change.
        #[css(iterable)]
        features: Box<[CustomIdent]>,
        /// A bitfield with the kind of change that the value will create, based
        /// on the above field.
        #[css(skip)]
        bits: WillChangeBits,
    },
}

impl WillChange {
    #[inline]
    /// Get default value of `will-change` as `auto`
    pub fn auto() -> WillChange {
        WillChange::Auto
    }
}

bitflags! {
    /// The change bits that we care about.
    ///
    /// These need to be in sync with NS_STYLE_WILL_CHANGE_*.
    #[derive(MallocSizeOf, SpecifiedValueInfo, ToComputedValue)]
    pub struct WillChangeBits: u8 {
        /// Whether the stacking context will change.
        const STACKING_CONTEXT = 1 << 0;
        /// Whether `transform` will change.
        const TRANSFORM = 1 << 1;
        /// Whether `scroll-position` will change.
        const SCROLL = 1 << 2;
        /// Whether `opacity` will change.
        const OPACITY = 1 << 3;
        /// Fixed pos containing block.
        const FIXPOS_CB = 1 << 4;
        /// Abs pos containing block.
        const ABSPOS_CB = 1 << 5;
    }
}

fn change_bits_for_longhand(longhand: LonghandId) -> WillChangeBits {
    let mut flags = match longhand {
        LonghandId::Opacity => WillChangeBits::OPACITY,
        LonghandId::Transform => WillChangeBits::TRANSFORM,
        _ => WillChangeBits::empty(),
    };

    let property_flags = longhand.flags();
    if property_flags.contains(PropertyFlags::CREATES_STACKING_CONTEXT) {
        flags |= WillChangeBits::STACKING_CONTEXT;
    }
    if property_flags.contains(PropertyFlags::FIXPOS_CB) {
        flags |= WillChangeBits::FIXPOS_CB;
    }
    if property_flags.contains(PropertyFlags::ABSPOS_CB) {
        flags |= WillChangeBits::ABSPOS_CB;
    }
    flags
}

fn change_bits_for_maybe_property(ident: &str, context: &ParserContext) -> WillChangeBits {
    let id = match PropertyId::parse_ignoring_rule_type(ident, context) {
        Ok(id) => id,
        Err(..) => return WillChangeBits::empty(),
    };

    match id.as_shorthand() {
        Ok(shorthand) => {
            shorthand.longhands().fold(WillChangeBits::empty(), |flags, p| {
                flags | change_bits_for_longhand(p)
            })
        }
        Err(PropertyDeclarationId::Longhand(longhand)) => change_bits_for_longhand(longhand),
        Err(PropertyDeclarationId::Custom(..)) => WillChangeBits::empty(),
    }
}

impl Parse for WillChange {
    /// auto | <animateable-feature>#
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<WillChange, ParseError<'i>> {
        if input
            .try(|input| input.expect_ident_matching("auto"))
            .is_ok()
        {
            return Ok(WillChange::Auto);
        }

        let mut bits = WillChangeBits::empty();
        let custom_idents = input.parse_comma_separated(|i| {
            let location = i.current_source_location();
            let parser_ident = i.expect_ident()?;
            let ident = CustomIdent::from_ident(
                location,
                parser_ident,
                &["will-change", "none", "all", "auto"],
            )?;

            if ident.0 == atom!("scroll-position") {
                bits |= WillChangeBits::SCROLL;
            } else {
                bits |= change_bits_for_maybe_property(&parser_ident, context);
            }
            Ok(ident)
        })?;

        Ok(WillChange::AnimateableFeatures {
            features: custom_idents.into_boxed_slice(),
            bits,
        })
    }
}

bitflags! {
    #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
    #[derive(SpecifiedValueInfo, ToComputedValue)]
    /// These constants match Gecko's `NS_STYLE_TOUCH_ACTION_*` constants.
    #[value_info(other_values = "auto,none,manipulation,pan-x,pan-y")]
    pub struct TouchAction: u8 {
        /// `none` variant
        const TOUCH_ACTION_NONE = 1 << 0;
        /// `auto` variant
        const TOUCH_ACTION_AUTO = 1 << 1;
        /// `pan-x` variant
        const TOUCH_ACTION_PAN_X = 1 << 2;
        /// `pan-y` variant
        const TOUCH_ACTION_PAN_Y = 1 << 3;
        /// `manipulation` variant
        const TOUCH_ACTION_MANIPULATION = 1 << 4;
    }
}

impl TouchAction {
    #[inline]
    /// Get default `touch-action` as `auto`
    pub fn auto() -> TouchAction {
        TouchAction::TOUCH_ACTION_AUTO
    }
}

impl ToCss for TouchAction {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            TouchAction::TOUCH_ACTION_NONE => dest.write_str("none"),
            TouchAction::TOUCH_ACTION_AUTO => dest.write_str("auto"),
            TouchAction::TOUCH_ACTION_MANIPULATION => dest.write_str("manipulation"),
            _ if self.contains(
                TouchAction::TOUCH_ACTION_PAN_X | TouchAction::TOUCH_ACTION_PAN_Y,
            ) =>
            {
                dest.write_str("pan-x pan-y")
            },
            _ if self.contains(TouchAction::TOUCH_ACTION_PAN_X) => dest.write_str("pan-x"),
            _ if self.contains(TouchAction::TOUCH_ACTION_PAN_Y) => dest.write_str("pan-y"),
            _ => panic!("invalid touch-action value"),
        }
    }
}

impl Parse for TouchAction {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<TouchAction, ParseError<'i>> {
        try_match_ident_ignore_ascii_case! { input,
            "auto" => Ok(TouchAction::TOUCH_ACTION_AUTO),
            "none" => Ok(TouchAction::TOUCH_ACTION_NONE),
            "manipulation" => Ok(TouchAction::TOUCH_ACTION_MANIPULATION),
            "pan-x" => {
                if input.try(|i| i.expect_ident_matching("pan-y")).is_ok() {
                    Ok(TouchAction::TOUCH_ACTION_PAN_X | TouchAction::TOUCH_ACTION_PAN_Y)
                } else {
                    Ok(TouchAction::TOUCH_ACTION_PAN_X)
                }
            },
            "pan-y" => {
                if input.try(|i| i.expect_ident_matching("pan-x")).is_ok() {
                    Ok(TouchAction::TOUCH_ACTION_PAN_X | TouchAction::TOUCH_ACTION_PAN_Y)
                } else {
                    Ok(TouchAction::TOUCH_ACTION_PAN_Y)
                }
            },
        }
    }
}

#[cfg(feature = "gecko")]
impl_bitflags_conversions!(TouchAction);

/// Asserts that all touch-action matches its NS_STYLE_TOUCH_ACTION_* value.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_touch_action_matches() {
    use gecko_bindings::structs;

    macro_rules! check_touch_action {
        ( $( $a:ident => $b:path),*, ) => {
            $(
                debug_assert_eq!(structs::$a as u8, $b.bits());
            )*
        }
    }

    check_touch_action! {
        NS_STYLE_TOUCH_ACTION_NONE => TouchAction::TOUCH_ACTION_NONE,
        NS_STYLE_TOUCH_ACTION_AUTO => TouchAction::TOUCH_ACTION_AUTO,
        NS_STYLE_TOUCH_ACTION_PAN_X => TouchAction::TOUCH_ACTION_PAN_X,
        NS_STYLE_TOUCH_ACTION_PAN_Y => TouchAction::TOUCH_ACTION_PAN_Y,
        NS_STYLE_TOUCH_ACTION_MANIPULATION => TouchAction::TOUCH_ACTION_MANIPULATION,
    }
}

bitflags! {
    #[derive(MallocSizeOf, SpecifiedValueInfo, ToComputedValue)]
    #[value_info(other_values = "none,strict,content,size,layout,style,paint")]
    /// Constants for contain: https://drafts.csswg.org/css-contain/#contain-property
    pub struct Contain: u8 {
        /// 'size' variant, turns on size containment
        const SIZE = 0x01;
        /// `layout` variant, turns on layout containment
        const LAYOUT = 0x02;
        /// `style` variant, turns on style containment
        const STYLE = 0x04;
        /// `paint` variant, turns on paint containment
        const PAINT = 0x08;
        /// `strict` variant, turns on all types of containment
        const STRICT = 0x10;
        /// 'content' variant, turns on style, layout, and paint containment
        const CONTENT = 0x20;
        /// variant with all the bits that contain: strict turns on
        const STRICT_BITS = Contain::LAYOUT.bits | Contain::STYLE.bits | Contain::PAINT.bits | Contain::SIZE.bits;
        /// variant with all the bits that contain: content turns on
        const CONTENT_BITS = Contain::STYLE.bits | Contain::LAYOUT.bits | Contain::PAINT.bits;
    }
}

impl ToCss for Contain {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_empty() {
            return dest.write_str("none");
        }
        if self.contains(Contain::STRICT) {
            return dest.write_str("strict");
        }
        if self.contains(Contain::CONTENT) {
            return dest.write_str("content");
        }

        let mut has_any = false;
        macro_rules! maybe_write_value {
            ($ident:path => $str:expr) => {
                if self.contains($ident) {
                    if has_any {
                        dest.write_str(" ")?;
                    }
                    has_any = true;
                    dest.write_str($str)?;
                }
            };
        }
        maybe_write_value!(Contain::SIZE => "size");
        maybe_write_value!(Contain::LAYOUT => "layout");
        maybe_write_value!(Contain::STYLE => "style");
        maybe_write_value!(Contain::PAINT => "paint");

        debug_assert!(has_any);
        Ok(())
    }
}

impl Parse for Contain {
    /// none | strict | content | [ size || layout || style || paint ]
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Contain, ParseError<'i>> {
        let mut result = Contain::empty();
        while let Ok(name) = input.try(|i| i.expect_ident_cloned()) {
            let flag = match_ignore_ascii_case! { &name,
                "size" => Some(Contain::SIZE),
                "layout" => Some(Contain::LAYOUT),
                "style" => Some(Contain::STYLE),
                "paint" => Some(Contain::PAINT),
                "strict" if result.is_empty() => return Ok(Contain::STRICT | Contain::STRICT_BITS),
                "content" if result.is_empty() => return Ok(Contain::CONTENT | Contain::CONTENT_BITS),
                "none" if result.is_empty() => return Ok(result),
                _ => None
            };

            let flag = match flag {
                Some(flag) if !result.contains(flag) => flag,
                _ => {
                    return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name)))
                },
            };
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(result)
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

/// A specified value for the `perspective` property.
pub type Perspective = GenericPerspective<NonNegativeLength>;

impl Parse for Perspective {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(GenericPerspective::None);
        }
        Ok(GenericPerspective::Length(NonNegativeLength::parse(
            context,
            input,
        )?))
    }
}

/// A given transition property, that is either `All`, a longhand or shorthand
/// property, or an unsupported or custom property.
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToComputedValue)]
pub enum TransitionProperty {
    /// A shorthand.
    Shorthand(ShorthandId),
    /// A longhand transitionable property.
    Longhand(LonghandId),
    /// A custom property.
    Custom(CustomPropertyName),
    /// Unrecognized property which could be any non-transitionable, custom property, or
    /// unknown property.
    Unsupported(CustomIdent),
}

impl ToCss for TransitionProperty {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        use values::serialize_atom_identifier;
        match *self {
            TransitionProperty::Shorthand(ref s) => s.to_css(dest),
            TransitionProperty::Longhand(ref l) => l.to_css(dest),
            TransitionProperty::Custom(ref name) => {
                dest.write_str("--")?;
                serialize_atom_identifier(name, dest)
            }
            TransitionProperty::Unsupported(ref i) => i.to_css(dest),
        }
    }
}

impl Parse for TransitionProperty {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let ident = input.expect_ident()?;

        let id = match PropertyId::parse_ignoring_rule_type(&ident, context) {
            Ok(id) => id,
            Err(..) => return Ok(TransitionProperty::Unsupported(
                CustomIdent::from_ident(location, ident, &["none"])?,
            )),
        };

        Ok(match id.as_shorthand() {
            Ok(s) => TransitionProperty::Shorthand(s),
            Err(longhand_or_custom) => {
                match longhand_or_custom {
                    PropertyDeclarationId::Longhand(id) => TransitionProperty::Longhand(id),
                    PropertyDeclarationId::Custom(custom) => {
                        TransitionProperty::Custom(custom.clone())
                    }
                }
            }
        })
    }
}

impl SpecifiedValueInfo for TransitionProperty {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        // `transition-property` can actually accept all properties and
        // arbitrary identifiers, but `all` is a special one we'd like
        // to list.
        f(&["all"]);
    }
}

impl TransitionProperty {
    /// Returns `all`.
    #[inline]
    pub fn all() -> Self {
        TransitionProperty::Shorthand(ShorthandId::All)
    }

    /// Convert TransitionProperty to nsCSSPropertyID.
    #[cfg(feature = "gecko")]
    pub fn to_nscsspropertyid(&self) -> Result<::gecko_bindings::structs::nsCSSPropertyID, ()> {
        Ok(match *self {
            TransitionProperty::Shorthand(ShorthandId::All) => {
                ::gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_all_properties
            }
            TransitionProperty::Shorthand(ref id) => id.to_nscsspropertyid(),
            TransitionProperty::Longhand(ref id) => id.to_nscsspropertyid(),
            TransitionProperty::Custom(..) |
            TransitionProperty::Unsupported(..) => return Err(()),
        })
    }
}
