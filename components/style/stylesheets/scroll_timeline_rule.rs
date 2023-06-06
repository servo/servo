/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! scroll-timeline-at-rule: https://drafts.csswg.org/scroll-animations/#scroll-timeline-at-rule

use crate::parser::{Parse, ParserContext};
use crate::shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::values::specified::{LengthPercentage, Number};
use crate::values::{AtomIdent, TimelineName};
use cssparser::{AtRuleParser, CowRcStr, DeclarationParser, Parser, SourceLocation, Token};
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Debug, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// A [`@scroll-timeline`][descriptors] rule.
///
/// [descriptors] https://drafts.csswg.org/scroll-animations/#scroll-timeline-descriptors
#[derive(Clone, Debug, ToShmem)]
pub struct ScrollTimelineRule {
    /// The name of the current scroll timeline.
    pub name: TimelineName,
    /// The descriptors.
    pub descriptors: ScrollTimelineDescriptors,
    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl ToCssWithGuard for ScrollTimelineRule {
    fn to_css(&self, _guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        let mut dest = CssWriter::new(dest);
        dest.write_str("@scroll-timeline ")?;
        self.name.to_css(&mut dest)?;
        dest.write_str(" { ")?;
        self.descriptors.to_css(&mut dest)?;
        dest.write_str("}")
    }
}

/// The descriptors of @scroll-timeline.
///
/// https://drafts.csswg.org/scroll-animations/#scroll-timeline-descriptors
#[derive(Clone, Debug, Default, ToShmem)]
pub struct ScrollTimelineDescriptors {
    /// The source of the current scroll timeline.
    pub source: Option<Source>,
    /// The orientation of the current scroll timeline.
    pub orientation: Option<Orientation>,
    /// The scroll timeline's scrollOffsets.
    pub offsets: Option<ScrollOffsets>,
}

impl Parse for ScrollTimelineDescriptors {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        use crate::cssparser::DeclarationListParser;
        use crate::error_reporting::ContextualParseError;

        let mut descriptors = ScrollTimelineDescriptors::default();
        let parser = ScrollTimelineDescriptorsParser {
            context,
            descriptors: &mut descriptors,
        };
        let mut iter = DeclarationListParser::new(input, parser);
        while let Some(declaration) = iter.next() {
            if let Err((error, slice)) = declaration {
                let location = error.location;
                let error = ContextualParseError::UnsupportedRule(slice, error);
                context.log_css_error(location, error)
            }
        }

        Ok(descriptors)
    }
}

// Basically, this is used for the serialization of CSSScrollTimelineRule, so we follow the
// instructions in https://drafts.csswg.org/scroll-animations-1/#serialize-a-cssscrolltimelinerule.
impl ToCss for ScrollTimelineDescriptors {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if let Some(ref value) = self.source {
            dest.write_str("source: ")?;
            value.to_css(dest)?;
            dest.write_str("; ")?;
        }

        if let Some(ref value) = self.orientation {
            dest.write_str("orientation: ")?;
            value.to_css(dest)?;
            dest.write_str("; ")?;
        }

        // https://github.com/w3c/csswg-drafts/issues/6617
        if let Some(ref value) = self.offsets {
            dest.write_str("scroll-offsets: ")?;
            value.to_css(dest)?;
            dest.write_str("; ")?;
        }
        Ok(())
    }
}

struct ScrollTimelineDescriptorsParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
    descriptors: &'a mut ScrollTimelineDescriptors,
}

impl<'a, 'b, 'i> AtRuleParser<'i> for ScrollTimelineDescriptorsParser<'a, 'b> {
    type Prelude = ();
    type AtRule = ();
    type Error = StyleParseErrorKind<'i>;
}

impl<'a, 'b, 'i> DeclarationParser<'i> for ScrollTimelineDescriptorsParser<'a, 'b> {
    type Declaration = ();
    type Error = StyleParseErrorKind<'i>;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<(), ParseError<'i>> {
        macro_rules! parse_descriptor {
            (
                $( $name: tt / $ident: ident, )*
            ) => {
                match_ignore_ascii_case! { &*name,
                    $(
                        $name => {
                            let value = input.parse_entirely(|i| Parse::parse(self.context, i))?;
                            self.descriptors.$ident = Some(value)
                        },
                    )*
                    _ => {
                        return Err(input.new_custom_error(
                            SelectorParseErrorKind::UnexpectedIdent(name.clone()),
                        ))
                    }
                }
            }
        }
        parse_descriptor! {
            "source" / source,
            "orientation" / orientation,
            "scroll-offsets" / offsets,
        };
        Ok(())
    }
}

/// The scroll-timeline source.
///
/// https://drafts.csswg.org/scroll-animations/#descdef-scroll-timeline-source
// FIXME: Bug 1733260 may drop the entire @scroll-timeline, and now we don't support source other
// than the default value (so use #[css(skip)]).
#[derive(Clone, Debug, Parse, PartialEq, ToCss, ToShmem)]
pub enum Source {
    /// The scroll container.
    #[css(skip)]
    Selector(ScrollTimelineSelector),
    /// The initial value. The scrollingElement of the Document associated with the Window that is
    /// the current global object.
    Auto,
    /// Null. However, it's not clear what is the expected behavior of this. See the spec issue:
    /// https://drafts.csswg.org/scroll-animations/#issue-0d1e73bd
    #[css(skip)]
    None,
}

impl Default for Source {
    fn default() -> Self {
        Source::Auto
    }
}

/// The scroll-timeline orientation.
/// https://drafts.csswg.org/scroll-animations/#descdef-scroll-timeline-orientation
///
/// Note: the initial orientation is auto, and we will treat it as block, the same as the
/// definition of ScrollTimelineOptions (WebIDL API).
/// https://drafts.csswg.org/scroll-animations/#dom-scrolltimelineoptions-orientation
#[derive(Clone, Copy, Debug, MallocSizeOf, Eq, Parse, PartialEq, PartialOrd, ToCss, ToShmem)]
#[repr(u8)]
pub enum ScrollDirection {
    /// The initial value.
    Auto,
    /// The direction along the block axis. This is the default value.
    Block,
    /// The direction along the inline axis
    Inline,
    /// The physical horizontal direction.
    Horizontal,
    /// The physical vertical direction.
    Vertical,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::Auto
    }
}

// Avoid name collision in cbindgen with StyleOrientation.
pub use self::ScrollDirection as Orientation;

/// Scroll-timeline offsets. We treat None as an empty vector.
/// value: none | <scroll-timeline-offset>#
///
/// https://drafts.csswg.org/scroll-animations/#descdef-scroll-timeline-scroll-offsets
#[derive(Clone, Default, Debug, ToCss, ToShmem)]
#[css(comma)]
pub struct ScrollOffsets(#[css(if_empty = "none", iterable)] Box<[ScrollTimelineOffset]>);

impl Parse for ScrollOffsets {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try_parse(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(ScrollOffsets(Box::new([])));
        }

        Ok(ScrollOffsets(
            input
                .parse_comma_separated(|i| ScrollTimelineOffset::parse(i))?
                .into_boxed_slice(),
        ))
    }
}

/// A <scroll-timeline-offset>.
/// value: auto | <length-percentage> | <element-offset>
///
/// https://drafts.csswg.org/scroll-animations/#typedef-scroll-timeline-offset
// FIXME: Bug 1733260 may drop the entire @scroll-timeline, and now we don't support
// <scroll-timeline-offset> other than the default value (so use #[css(skip)]).
#[derive(Clone, Debug, Parse, PartialEq, ToCss, ToShmem)]
pub enum ScrollTimelineOffset {
    /// The initial value. A container-based offset.
    Auto,
    /// A container-based offset with the distance indicated by the value along source's scroll
    /// range in orientation.
    #[css(skip)]
    LengthPercentage(LengthPercentage),
    /// An element-based offset.
    #[css(skip)]
    ElementOffset(ElementOffset),
}

/// An <element-offset-edge>.
///
/// https://drafts.csswg.org/scroll-animations-1/#typedef-element-offset-edge
#[derive(Clone, Copy, Debug, MallocSizeOf, Eq, Parse, PartialEq, PartialOrd, ToCss, ToShmem)]
pub enum ElementOffsetEdge {
    /// Start edge
    Start,
    /// End edge.
    End,
}

/// An <element-offset>.
/// value: selector( <id-selector> ) [<element-offset-edge> || <number>]?
///
/// https://drafts.csswg.org/scroll-animations-1/#typedef-element-offset
#[derive(Clone, Debug, PartialEq, ToCss, ToShmem)]
pub struct ElementOffset {
    /// The target whose intersection with source's scrolling box determines the concrete scroll
    /// offset.
    target: ScrollTimelineSelector,
    /// An optional value of <element-offset-edge>. If not provided, the default value is start.
    edge: Option<ElementOffsetEdge>,
    /// An optional value of threshold. If not provided, the default value is 0.
    threshold: Option<Number>,
}

impl Parse for ElementOffset {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let target = ScrollTimelineSelector::parse(context, input)?;

        // Parse `[<element-offset-edge> || <number>]?`
        let mut edge = input.try_parse(ElementOffsetEdge::parse).ok();
        let threshold = input.try_parse(|i| Number::parse(context, i)).ok();
        if edge.is_none() {
            edge = input.try_parse(ElementOffsetEdge::parse).ok();
        }

        Ok(ElementOffset {
            target,
            edge,
            threshold,
        })
    }
}

/// The type of the selector ID.
#[derive(Clone, Eq, PartialEq, ToShmem)]
pub struct ScrollTimelineSelector(AtomIdent);

impl Parse for ScrollTimelineSelector {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // Parse `selector(<id-selector>)`.
        input.expect_function_matching("selector")?;
        input.parse_nested_block(|i| match i.next()? {
            Token::IDHash(id) => Ok(ScrollTimelineSelector(id.as_ref().into())),
            _ => Err(i.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
        })
    }
}

impl ToCss for ScrollTimelineSelector {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("selector(")?;
        dest.write_char('#')?;
        self.0.to_css(dest)?;
        dest.write_char(')')
    }
}

impl Debug for ScrollTimelineSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_css(&mut CssWriter::new(f))
    }
}
