/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

${helpers.two_properties_shorthand(
    "overflow",
    "overflow-x",
    "overflow-y",
    engines="gecko servo",
    flags="SHORTHAND_IN_GETCS",
    spec="https://drafts.csswg.org/css-overflow/#propdef-overflow",
)}

${helpers.two_properties_shorthand(
    "overflow-clip-box",
    "overflow-clip-box-block",
    "overflow-clip-box-inline",
    engines="gecko",
    enabled_in="ua",
    gecko_pref="layout.css.overflow-clip-box.enabled",
    spec="Internal, may be standardized in the future "
         "(https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box)",
)}

macro_rules! try_parse_one {
    ($context: expr, $input: expr, $var: ident, $prop_module: ident) => {
        if $var.is_none() {
            if let Ok(value) = $input.try_parse(|i| {
                $prop_module::single_value::parse($context, i)
            }) {
                $var = Some(value);
                continue;
            }
        }
    };
}

<%helpers:shorthand name="transition"
                    engines="gecko servo"
                    extra_prefixes="moz:layout.css.prefixes.transitions webkit"
                    sub_properties="transition-property transition-duration
                                    transition-timing-function
                                    transition-delay"
                    spec="https://drafts.csswg.org/css-transitions/#propdef-transition">
    use crate::parser::Parse;
    % for prop in "delay duration property timing_function".split():
    use crate::properties::longhands::transition_${prop};
    % endfor
    use crate::values::specified::TransitionProperty;

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        struct SingleTransition {
            % for prop in "duration timing_function delay".split():
            transition_${prop}: transition_${prop}::SingleSpecifiedValue,
            % endfor
            // Unlike other properties, transition-property uses an Option<> to
            // represent 'none' as `None`.
            transition_property: Option<TransitionProperty>,
        }

        fn parse_one_transition<'i, 't>(
            context: &ParserContext,
            input: &mut Parser<'i, 't>,
        ) -> Result<SingleTransition,ParseError<'i>> {
            % for prop in "property duration timing_function delay".split():
            let mut ${prop} = None;
            % endfor

            let mut parsed = 0;
            loop {
                parsed += 1;

                try_parse_one!(context, input, duration, transition_duration);
                try_parse_one!(context, input, timing_function, transition_timing_function);
                try_parse_one!(context, input, delay, transition_delay);
                // Must check 'transition-property' after 'transition-timing-function' since
                // 'transition-property' accepts any keyword.
                if property.is_none() {
                    if let Ok(value) = input.try_parse(|i| TransitionProperty::parse(context, i)) {
                        property = Some(Some(value));
                        continue;
                    }

                    if input.try_parse(|i| i.expect_ident_matching("none")).is_ok() {
                        // 'none' is not a valid value for <single-transition-property>,
                        // so it's not acceptable in the function above.
                        property = Some(None);
                        continue;
                    }
                }

                parsed -= 1;
                break
            }

            if parsed != 0 {
                Ok(SingleTransition {
                    % for prop in "duration timing_function delay".split():
                    transition_${prop}: ${prop}.unwrap_or_else(transition_${prop}::single_value
                                                                                 ::get_initial_specified_value),
                    % endfor
                    transition_property: property.unwrap_or(
                        Some(transition_property::single_value::get_initial_specified_value())),
                })
            } else {
                Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            }
        }

        % for prop in "property duration timing_function delay".split():
        let mut ${prop}s = Vec::new();
        % endfor

        let results = input.parse_comma_separated(|i| parse_one_transition(context, i))?;
        let multiple_items = results.len() >= 2;
        for result in results {
            if let Some(value) = result.transition_property {
                propertys.push(value);
            } else if multiple_items {
                // If there is more than one item, and any of transitions has 'none',
                // then it's invalid. Othersize, leave propertys to be empty (which
                // means "transition-property: none");
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }

            % for prop in "duration timing_function delay".split():
            ${prop}s.push(result.transition_${prop});
            % endfor
        }

        Ok(expanded! {
            % for prop in "property duration timing_function delay".split():
            transition_${prop}: transition_${prop}::SpecifiedValue(${prop}s.into()),
            % endfor
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            let property_len = self.transition_property.0.len();

            // There are two cases that we can do shorthand serialization:
            // * when all value lists have the same length, or
            // * when transition-property is none, and other value lists have exactly one item.
            if property_len == 0 {
                % for name in "duration delay timing_function".split():
                    if self.transition_${name}.0.len() != 1 {
                        return Ok(());
                    }
                % endfor
            } else {
                % for name in "duration delay timing_function".split():
                    if self.transition_${name}.0.len() != property_len {
                        return Ok(());
                    }
                % endfor
            }

            // Representative length.
            let len = self.transition_duration.0.len();

            for i in 0..len {
                if i != 0 {
                    dest.write_str(", ")?;
                }
                if property_len == 0 {
                    dest.write_str("none")?;
                } else {
                    self.transition_property.0[i].to_css(dest)?;
                }
                % for name in "duration timing_function delay".split():
                    dest.write_str(" ")?;
                    self.transition_${name}.0[i].to_css(dest)?;
                % endfor
            }
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="animation"
                    engines="gecko servo"
                    extra_prefixes="moz:layout.css.prefixes.animations webkit"
                    sub_properties="animation-name animation-duration
                                    animation-timing-function animation-delay
                                    animation-iteration-count animation-direction
                                    animation-fill-mode animation-play-state animation-timeline"
                    rule_types_allowed="Style"
                    spec="https://drafts.csswg.org/css-animations/#propdef-animation">
    <%
        props = "name timeline duration timing_function delay iteration_count \
                 direction fill_mode play_state".split()
    %>
    % for prop in props:
    use crate::properties::longhands::animation_${prop};
    % endfor

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        struct SingleAnimation {
            % for prop in props:
            animation_${prop}: animation_${prop}::SingleSpecifiedValue,
            % endfor
        }

        fn scroll_linked_animations_enabled() -> bool {
            #[cfg(feature = "gecko")]
            return static_prefs::pref!("layout.css.scroll-linked-animations.enabled");
            #[cfg(feature = "servo")]
            return false;
        }

        fn parse_one_animation<'i, 't>(
            context: &ParserContext,
            input: &mut Parser<'i, 't>,
        ) -> Result<SingleAnimation, ParseError<'i>> {
            % for prop in props:
            let mut ${prop} = None;
            % endfor

            let mut parsed = 0;
            // NB: Name must be the last one here so that keywords valid for other
            // longhands are not interpreted as names.
            //
            // Also, duration must be before delay, see
            // https://drafts.csswg.org/css-animations/#typedef-single-animation
            loop {
                parsed += 1;
                try_parse_one!(context, input, duration, animation_duration);
                try_parse_one!(context, input, timing_function, animation_timing_function);
                try_parse_one!(context, input, delay, animation_delay);
                try_parse_one!(context, input, iteration_count, animation_iteration_count);
                try_parse_one!(context, input, direction, animation_direction);
                try_parse_one!(context, input, fill_mode, animation_fill_mode);
                try_parse_one!(context, input, play_state, animation_play_state);
                try_parse_one!(context, input, name, animation_name);
                if scroll_linked_animations_enabled() {
                    try_parse_one!(context, input, timeline, animation_timeline);
                }

                parsed -= 1;
                break
            }

            // If nothing is parsed, this is an invalid entry.
            if parsed == 0 {
                Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            } else {
                Ok(SingleAnimation {
                    % for prop in props:
                    animation_${prop}: ${prop}.unwrap_or_else(animation_${prop}::single_value
                                                              ::get_initial_specified_value),
                    % endfor
                })
            }
        }

        % for prop in props:
        let mut ${prop}s = vec![];
        % endfor

        let results = input.parse_comma_separated(|i| parse_one_animation(context, i))?;
        for result in results.into_iter() {
            % for prop in props:
            ${prop}s.push(result.animation_${prop});
            % endfor
        }

        Ok(expanded! {
            % for prop in props:
            animation_${prop}: animation_${prop}::SpecifiedValue(${prop}s.into()),
            % endfor
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            let len = self.animation_name.0.len();
            // There should be at least one declared value
            if len == 0 {
                return Ok(());
            }

            // If any value list length is differs then we don't do a shorthand serialization
            // either.
            % for name in props[2:]:
                if len != self.animation_${name}.0.len() {
                    return Ok(())
                }
            % endfor

            // If the preference of animation-timeline is disabled, `self.animation_timeline` is
            // None.
            if self.animation_timeline.map_or(false, |v| len != v.0.len()) {
                return Ok(());
            }

            for i in 0..len {
                if i != 0 {
                    dest.write_str(", ")?;
                }

                % for name in props[2:]:
                    self.animation_${name}.0[i].to_css(dest)?;
                    dest.write_str(" ")?;
                % endfor

                self.animation_name.0[i].to_css(dest)?;

                // Based on the spec, the default values of other properties must be output in at
                // least the cases necessary to distinguish an animation-name. The serialization
                // order of animation-timeline is always later than animation-name, so it's fine
                // to not serialize it if it is the default value. It's still possible to
                // distinguish them (because we always serialize animation-name).
                // https://drafts.csswg.org/css-animations-1/#animation
                // https://drafts.csswg.org/css-animations-2/#typedef-single-animation
                //
                // Note: it's also fine to always serialize this. However, it seems Blink
                // doesn't serialize default animation-timeline now, so we follow the same rule.
                if let Some(ref timeline) = self.animation_timeline {
                    if !timeline.0[i].is_auto() {
                        dest.write_char(' ')?;
                        timeline.0[i].to_css(dest)?;
                    }
                }
            }
            Ok(())
        }
    }
</%helpers:shorthand>

${helpers.two_properties_shorthand(
    "overscroll-behavior",
    "overscroll-behavior-x",
    "overscroll-behavior-y",
    engines="gecko",
    gecko_pref="layout.css.overscroll-behavior.enabled",
    spec="https://wicg.github.io/overscroll-behavior/#overscroll-behavior-properties",
)}

<%helpers:shorthand
    engines="gecko"
    name="container"
    sub_properties="container-type container-name"
    gecko_pref="layout.css.container-queries.enabled",
    spec="https://drafts.csswg.org/css-contain-3/#container-shorthand"
>
    pub fn parse_value<'i>(
        context: &ParserContext,
        input: &mut Parser<'i, '_>,
    ) -> Result<Longhands, ParseError<'i>> {
        use crate::parser::Parse;
        use crate::values::specified::box_::{ContainerName, ContainerType};
        // See https://github.com/w3c/csswg-drafts/issues/7180 for why we don't
        // match the spec.
        let container_type = ContainerType::parse(context, input)?;
        let container_name = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            ContainerName::parse(context, input)?
        } else {
            ContainerName::none()
        };
        Ok(expanded! {
            container_type: container_type,
            container_name: container_name,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            self.container_type.to_css(dest)?;
            if !self.container_name.is_none() {
                dest.write_str(" / ")?;
                self.container_name.to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand
    engines="gecko"
    name="page-break-before"
    flags="SHORTHAND_IN_GETCS IS_LEGACY_SHORTHAND"
    sub_properties="break-before"
    spec="https://drafts.csswg.org/css-break-3/#page-break-properties"
>
    pub fn parse_value<'i>(
        context: &ParserContext,
        input: &mut Parser<'i, '_>,
    ) -> Result<Longhands, ParseError<'i>> {
        use crate::values::specified::box_::BreakBetween;
        Ok(expanded! {
            break_before: BreakBetween::parse_legacy(context, input)?,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            self.break_before.to_css_legacy(dest)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand
    engines="gecko"
    name="page-break-after"
    flags="SHORTHAND_IN_GETCS IS_LEGACY_SHORTHAND"
    sub_properties="break-after"
    spec="https://drafts.csswg.org/css-break-3/#page-break-properties"
>
    pub fn parse_value<'i>(
        context: &ParserContext,
        input: &mut Parser<'i, '_>,
    ) -> Result<Longhands, ParseError<'i>> {
        use crate::values::specified::box_::BreakBetween;
        Ok(expanded! {
            break_after: BreakBetween::parse_legacy(context, input)?,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            self.break_after.to_css_legacy(dest)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand
    engines="gecko"
    name="page-break-inside"
    flags="SHORTHAND_IN_GETCS IS_LEGACY_SHORTHAND"
    sub_properties="break-inside"
    spec="https://drafts.csswg.org/css-break-3/#page-break-properties"
>
    pub fn parse_value<'i>(
        context: &ParserContext,
        input: &mut Parser<'i, '_>,
    ) -> Result<Longhands, ParseError<'i>> {
        use crate::values::specified::box_::BreakWithin;
        Ok(expanded! {
            break_inside: BreakWithin::parse_legacy(context, input)?,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            self.break_inside.to_css_legacy(dest)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="offset"
                    engines="gecko"
                    sub_properties="offset-path offset-distance offset-rotate offset-anchor"
                    gecko_pref="layout.css.motion-path.enabled",
                    spec="https://drafts.fxtf.org/motion-1/#offset-shorthand">
    use crate::parser::Parse;
    use crate::values::specified::motion::{OffsetPath, OffsetRotate};
    use crate::values::specified::position::PositionOrAuto;
    use crate::values::specified::LengthPercentage;
    use crate::Zero;

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        // FIXME: Bug 1559232: Support offset-position.
        // Per the spec, this must have offet-position and/or offset-path. However, we don't
        // support offset-position, so offset-path is necessary now.
        let offset_path = OffsetPath::parse(context, input)?;

        let mut offset_distance = None;
        let mut offset_rotate = None;
        loop {
            if offset_distance.is_none() {
                if let Ok(value) = input.try_parse(|i| LengthPercentage::parse(context, i)) {
                    offset_distance = Some(value);
                }
            }

            if offset_rotate.is_none() {
                if let Ok(value) = input.try_parse(|i| OffsetRotate::parse(context, i)) {
                    offset_rotate = Some(value);
                    continue;
                }
            }
            break;
        }

        let offset_anchor = input.try_parse(|i| {
            i.expect_delim('/')?;
            PositionOrAuto::parse(context, i)
        }).ok();

        Ok(expanded! {
            offset_path: offset_path,
            offset_distance: offset_distance.unwrap_or(LengthPercentage::zero()),
            offset_rotate: offset_rotate.unwrap_or(OffsetRotate::auto()),
            offset_anchor: offset_anchor.unwrap_or(PositionOrAuto::auto()),
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            // FIXME: Bug 1559232: Support offset-position. We don't support offset-position,
            // so always serialize offset-path now.
            self.offset_path.to_css(dest)?;

            if !self.offset_distance.is_zero() {
                dest.write_str(" ")?;
                self.offset_distance.to_css(dest)?;
            }

            if !self.offset_rotate.is_auto() {
                dest.write_str(" ")?;
                self.offset_rotate.to_css(dest)?;
            }

            if *self.offset_anchor != PositionOrAuto::auto() {
                dest.write_str(" / ")?;
                self.offset_anchor.to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="zoom" engines="gecko"
                    sub_properties="transform transform-origin"
                    gecko_pref="layout.css.zoom-transform-hack.enabled"
                    flags="SHORTHAND_IN_GETCS IS_LEGACY_SHORTHAND"
                    spec="Not a standard, only a compat hack">
    use crate::parser::Parse;
    use crate::values::specified::{Number, NumberOrPercentage, TransformOrigin};
    use crate::values::generics::transform::{Transform, TransformOperation};

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let zoom = match input.try_parse(|input| NumberOrPercentage::parse(context, input)) {
            Ok(number_or_percent) => number_or_percent.to_number(),
            Err(..) => {
                input.expect_ident_matching("normal")?;
                Number::new(1.0)
            },
        };

        // Make sure that the initial value matches the values for the
        // longhands, just for general sanity.  `zoom: 1` and `zoom: 0` are
        // ignored, see [1][2]. They are just hack for the "has layout" mode on
        // IE.
        //
        // [1]: https://bugs.webkit.org/show_bug.cgi?id=18467
        // [2]: https://bugzilla.mozilla.org/show_bug.cgi?id=1593009
        Ok(if zoom.get() == 1.0 || zoom.get() == 0.0 {
            expanded! {
                transform: Transform::none(),
                transform_origin: TransformOrigin::initial_value(),
            }
        } else {
            expanded! {
                transform: Transform(vec![TransformOperation::Scale(zoom, zoom)].into()),
                transform_origin: TransformOrigin::zero_zero(),
            }
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            if self.transform.0.is_empty() && *self.transform_origin == TransformOrigin::initial_value() {
                return 1.0f32.to_css(dest);
            }
            if *self.transform_origin != TransformOrigin::zero_zero() {
                return Ok(())
            }
            match &*self.transform.0 {
                [TransformOperation::Scale(x, y)] if x == y => x.to_css(dest),
                _ => Ok(()),
            }
        }
    }
</%helpers:shorthand>
