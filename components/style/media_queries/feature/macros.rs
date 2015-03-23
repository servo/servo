/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

macro_rules! media_features {
    ($($feature:ident(name: $name:expr,
                      $($rest:tt)*)),+) => {
        #[derive(Copy, Debug, PartialEq)]
        pub enum MediaFeature {
            $($feature(Option<$feature>)),+
        }

        //trace_macros!(true);
        $(media_feature!($feature, $($rest)*);)+
        //trace_macros!(false);

        derive_dispatch_fns!($($name => $feature),+);
        derive_feature_context_trait!($($feature),+);
    }
}

macro_rules! media_feature {
    // discrete feature rules
    ($feature:ident,
     value: { $($css:expr => $variant:ident),+ },
     type: discrete,
     availability: { $($availability:tt)+ }) =>
    {
        discrete!($feature,
                  value: { $($css => $variant),+ },
                  availability: { $($availability)+ });
    };
    ($feature:ident,
     value: mq_boolean,
     type: discrete,
     availability: { $($availability:tt)+ }) =>
    {
        discrete!($feature,
                  value: mq_boolean,
                  availability: { $($availability)+ });
    };

    // range feature rules
    ($variant:ident,
     value: $value:ty,
     type: range,
     availability: { $($availability:tt)+ },
     impl: { $($impl_:tt)+ }) =>
    {
        range!($variant,
               value: $value,
               availability: { $($availability)+ },
               impl: { $($impl_)+ });
    };
}

macro_rules! discrete {
    // following two rules are the variations on the main form of the discrete! macro
    ($feature:ident,
     value: { $($css:expr => $variant:ident),+ },
     availability: { $($availability:tt)+ }) =>
    {
        discrete!(enum $feature { $($variant),+ });

        discrete!(derive to_media_feature_css for $feature);
        discrete!(derive ToCss,FromCss for $feature, $($css => $variant),+);
        discrete!(derive Parse for $feature, $($availability)+);
        discrete!(derive EvaluateUsingContextValue<ContextValue=$feature> for $feature, $($variant),+);
    };

    ($feature:ident,
     value: mq_boolean,
     availability: { $($availability:tt)+ }) =>
    {
        #[derive(Copy, Debug, PartialEq, Eq)]
        pub struct $feature(pub bool);

        discrete!(derive to_media_feature_css for $feature);
        discrete!(derive Parse for $feature, $($availability)+);

        impl PartialEq<bool> for $feature {
            #[inline]
            fn eq(&self, other: &bool) -> bool {
                self.0 == *other
            }
        }

        impl FromCss for $feature {
            type Err = ();
            type Context = SpecificationLevel;

            fn from_css(input: &mut Parser, _: &SpecificationLevel) -> Result<$feature, ()> {
                match try!(input.expect_integer()) {
                    0 => Ok($feature(false)),
                    1 => Ok($feature(true)),
                    _ => Err(())
                }
            }
        }

        impl ToCss for $feature {
            fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
                where W: ::text_writer::TextWriter
            {
                match self.0 {
                    false => dest.write_str("0"),
                    true => dest.write_str("1"),
                }
            }
        }

        impl ::std::fmt::Display for $feature {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                use ::cssparser::ToCss;

                self.fmt_to_css(f)
            }
        }

        impl<C> EvaluateUsingContextValue<C> for $feature
            where C: DeviceFeatureContext
        {
            type ContextValue = bool;

            fn evaluate(feature_value: &Option<Self>, _: &C, context_value: bool) -> bool {
                match feature_value {
                    &Some(ref feature_value) => *feature_value == context_value,
                    &None => context_value != false
                }
            }
        }
    };

    // internal rules below
    (enum $feature:ident { $($variant:ident),+ }) => {
        #[derive(Copy, Debug, PartialEq, Eq)]
        pub enum $feature {
            $($variant),+
        }
    };

    (derive Parse for $feature:ident, since: $since:expr) => {
        impl $feature {
            #[inline]
            fn parse_ident_first_form(input: &mut Parser,
                                      level: &SpecificationLevel,
                                      prefix: Option<RangePrefix>)
                                      -> Result<MediaFeature, ()>
            {
                if *level < $since {
                    return Err(())
                }

                parse_discrete(input, level, prefix).map(MediaFeature::$feature)
            }

            #[inline]
            fn parse_value_first_form(_: &mut Parser,
                                      _: &SpecificationLevel,
                                      _: SourcePosition)
                                      -> Result<MediaFeature, ()> {
                Err(())
            }
        }
    };
    (derive Parse for $feature:ident, since: $since:expr, deprecated: $deprecated:expr) => {
        // currently, deprecation has no effect for media features
        discrete!(derive Parse for $feature, since: $since);
    };

    (derive to_media_feature_css for $feature:ident) => {
        impl $feature {
            fn to_media_feature_css<W>(&self, dest: &mut W, name: &'static str) -> ::text_writer::Result
                where W: ::text_writer::TextWriter
            {
                try!(write!(dest, "({}: ", name));
                try!(self.to_css(dest));
                write!(dest, ")")
            }
        }
    };

    (derive ToCss,FromCss for $feature:ident, $($css:expr => $variant:ident),+) => {
        impl FromCss for $feature {
            type Err = ();
            type Context = SpecificationLevel;

            fn from_css(input: &mut Parser, _: &SpecificationLevel) -> Result<$feature, ()> {
                match &try!(input.expect_ident()) {
                    $(s if s.eq_ignore_ascii_case($css) => Ok($feature::$variant)),+,
                    _ => Err(())
                }
            }
        }

        impl ToCss for $feature {
            fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
                where W: ::text_writer::TextWriter
            {
                match self {
                    $(&$feature::$variant => dest.write_str($css)),+
                }
            }
        }

        impl ::std::fmt::Display for $feature {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                use ::cssparser::ToCss;

                self.fmt_to_css(f)
            }
        }
    };

    // we can use recursion to determine if the values contains 'None' or not
    // base cases
    (derive EvaluateUsingContextValue<ContextValue=$context:ty> for $feature:ident, None, $($rest:tt)* ) => {
        impl<C> EvaluateUsingContextValue<C> for $feature
            where C: DeviceFeatureContext
        {
            type ContextValue = $context;

            fn evaluate(feature_value: &Option<Self>, _: &C, context_value: $context) -> bool {
                match feature_value {
                    &Some(ref feature_value) => *feature_value == context_value,
                    &None => context_value != $feature::None
                }
            }
        }
    };
    (derive EvaluateUsingContextValue<ContextValue=$context:ty> for $feature:ident, ) => {
        impl<C> EvaluateUsingContextValue<C> for $feature
            where C: DeviceFeatureContext
        {
            type ContextValue = Option<$context>;

            fn evaluate(feature_value: &Option<Self>, _: &C, context_value: Option<$context>) -> bool {
                if let Some(context_value) = context_value {
                    match feature_value {
                        &Some(ref feature_value) => *feature_value == context_value,
                        &None => true
                    }
                } else {
                    false
                }
            }
        }
    };

    // reduction steps
    (derive EvaluateUsingContextValue<ContextValue=$context:ty> for $feature:ident, $variant:ident, $($rest:tt)*) => {
        discrete!(derive EvaluateUsingContextValue<ContextValue=$context> for $feature, $($rest)*);
    };
    (derive EvaluateUsingContextValue<ContextValue=$context:ty> for $feature:ident, $variant:ident) => {
        discrete!(derive EvaluateUsingContextValue<ContextValue=$context> for $feature, );
    };
}

macro_rules! range {
    // following two rules are the variations on the main form of the range! macro
    ($feature:ident,
     value: $value:ty,
     availability: { $($availability:tt)+ },
     impl: { context: $context:ty, zero: $zero:expr }) =>
    {
        range!(struct $feature($value));
        range!(derive to_media_feature_css for $feature);
        range!(derive Parse for $feature, $($availability)+);
        range!(derive EvaluateUsingContextValue<ContextValue=$context> for $feature,
               Zero=$zero);
    };
    ($feature:ident,
     value: $value:ty,
     availability: { $($availability:tt)+ },
     impl: { context: $context:ty, zero: $zero:expr, compute: $compute:ident }) =>
    {
        range!(struct $feature($value));
        range!(derive to_media_feature_css for $feature);
        range!(derive Parse for $feature, $($availability)+);
        range!(derive EvaluateUsingContextValue<ContextValue=$context> for $feature,
               Zero=$zero, Compute=$compute);
    };

    // internal rules below
    (struct $feature:ident($value:ty)) => {
        #[derive(Copy, Debug, PartialEq)]
        pub struct $feature(pub Range<$value>);
    };

    (derive to_media_feature_css for $feature:ident) => {
        impl $feature {
            fn to_media_feature_css<W>(&self, dest: &mut W, name: &'static str) -> ::text_writer::Result
                where W: ::text_writer::TextWriter
            {
                try!(write!(dest, "("));
                try!(self.0.to_css(dest, name));
                write!(dest, ")")
            }
        }
    };

    (derive Parse for $feature:ident, since: $since:expr) => {
        impl $feature {
            #[inline]
            fn parse_ident_first_form(input: &mut Parser,
                                      level: &SpecificationLevel,
                                      prefix: Option<RangePrefix>)
                                      -> Result<MediaFeature, ()>
            {
                if *level < $since {
                    return Err(())
                }

                parse_boolean_or_normal_range(input, level, prefix)
                    .map(|opt_range| opt_range.map(|range| $feature(range)))
                    .map(MediaFeature::$feature)
            }

            #[inline]
            fn parse_value_first_form(input: &mut Parser,
                                      level: &SpecificationLevel,
                                      after_name: SourcePosition)
                                      -> Result<MediaFeature, ()>
            {
                if *level < $since {
                    return Err(())
                }

                parse_range_form(input, level, after_name)
                    .map(|range| Some($feature(range)))
                    .map(MediaFeature::$feature)
            }
        }
    };
    (derive Parse for $feature:ident, since: $since:expr, deprecated: $deprecated:expr) => {
        // currently, deprecation has no effect for media features
        range!(derive Parse for $feature, since: $since);
    };

    (derive EvaluateUsingContextValue<ContextValue=$context:ty> for $feature:ident,
     Zero=$zero:expr) => {
        impl<C> EvaluateUsingContextValue<C> for $feature
            where C: DeviceFeatureContext
        {
            type ContextValue = $context;

            fn evaluate(feature_value: &Option<Self>, _: &C, context_value: $context) -> bool {
                match feature_value {
                    &Some(ref feature_value) => feature_value.0.evaluate(context_value),
                    &None => context_value != $zero
                }
            }
        }
    };
    (derive EvaluateUsingContextValue<ContextValue=$context:ty> for $feature:ident,
     Zero=$zero:expr, Compute=$compute:ident) => {
        impl<C> EvaluateUsingContextValue<C> for $feature
            where C: DeviceFeatureContext
        {
            type ContextValue = $context;

            fn evaluate(feature_value: &Option<Self>, context: &C, context_value: $context) -> bool {
                match feature_value {
                    &Some(ref feature_value) =>
                        feature_value.0.map(|v| $compute(v, context)).evaluate(context_value),
                    &None => context_value != $zero
                }
            }
        }
    };
}

macro_rules! derive_dispatch_fns {
    ($($css:expr => $feature:ident),+) => {
        #[inline]
        fn dispatch_to_css<W>(feature: &MediaFeature, dest: &mut W) -> ::text_writer::Result
            where W: ::text_writer::TextWriter
        {
            match feature {
                $(&MediaFeature::$feature(None) =>
                      write!(dest, "({})", $css),
                  &MediaFeature::$feature(Some(ref value)) =>
                      value.to_media_feature_css(dest, $css)),+
            }
        }

        #[inline]
        fn dispatch_parse_ident_first_form<'a>(input: &mut Parser,
                                               level: &SpecificationLevel,
                                               prefix: Option<RangePrefix>,
                                               name: &'a str)
                                               -> Result<MediaFeature, ()>
        {
            match name {
                $(n if $css.eq_ignore_ascii_case(n) =>
                      $feature::parse_ident_first_form(input, level, prefix),)+
                _ => Err(())
            }
        }

        #[inline]
        fn dispatch_parse_value_first_form<'a>(input: &mut Parser,
                                               level: &SpecificationLevel,
                                               name: &'a str,
                                               after_name: SourcePosition)
                                               -> Result<MediaFeature, ()>
        {
            match name {
                $(n if $css.eq_ignore_ascii_case(n) =>
                      $feature::parse_value_first_form(input, level, after_name),)+
                _ => Err(())
            }
        }

        #[inline]
        fn dispatch_evaluate<C>(feature: &MediaFeature, context: &C) -> bool
            where C: DeviceFeatureContext
        {
            match feature {
                $(&MediaFeature::$feature(ref opt_value) =>
                      $feature::evaluate(opt_value, context, context.$feature())),+
            }
        }
    }
}

macro_rules! derive_feature_context_trait {
    ($($feature:ident),+) => {
        #[allow(non_snake_case)]
        pub trait DeviceFeatureContext {
            fn MediaType(&self) -> ::media_queries::MediaType;
            fn ViewportSize(&self) -> Size2D<Au>;

            $(fn $feature(&self) -> <$feature as EvaluateUsingContextValue<Self>>::ContextValue;)+
        }
    }
}
