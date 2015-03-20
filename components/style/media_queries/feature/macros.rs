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

        $(media_feature!($feature, $($rest)*);)+

        derive_dispatch_fns!($($name => $feature),+);
        derive_feature_context_trait!($($feature),+);
    }
}

macro_rules! media_feature {
    ($feature:ident,
     value: mq_boolean,
     type: discrete,
     availability: {
         $($availability:tt)+
     }) => {
        discrete!($feature, value: mq_boolean);
    };

    ($feature:ident,
     value: { $($css:expr => $variant:ident),+ },
     type: discrete,
     availability: {
         $($availability:tt)+
     }) => {
        discrete!($feature, value: { $($css => $variant),+ });
    };
    ($feature:ident,
     value: { $($css:expr => $variant:ident),+ },
     type: discrete,
     availability: {
         $($availability:tt)+
     },
     impl: {
         $($impl_rest:tt)+
     }) => {
        discrete!($feature, value: { $($css => $variant),+ }, $($impl_rest)+);
    };

    ($variant:ident,
     value: $value:ty,
     type: range,
     availability: {
         $($availability:tt)+
     },
     impl: {
         $($impl_rest:tt)+
     }) => {
        range!($variant,
               value: $value,
               impl: {
                   $($impl_rest)+
               });
    };
}

macro_rules! discrete {
    (enum $feature:ident { $($variant:ident),+ }) => {
        #[derive(Copy, Debug, PartialEq, Eq)]
        pub enum $feature {
            $($variant),+
        }
    };

    (derive Parse for $feature:ident) => {
        impl $feature {
            #[inline]
            fn parse_ident_first_form(input: &mut Parser,
                                      prefix: Option<RangePrefix>)
                                      -> Result<MediaFeature, ()>
            {
                parse_discrete(input, prefix).map(MediaFeature::$feature)
            }

            #[inline]
            fn parse_value_first_form(_: &mut Parser,
                                      _: SourcePosition)
                                      -> Result<MediaFeature, ()> {
                Err(())
            }
        }
    };

    (derive to_media_feature_css for $feature:ident) => {
        impl $feature {
            fn to_media_feature_css<W>(&self, dest: &mut W, css: &'static str) -> ::text_writer::Result
                where W: ::text_writer::TextWriter
            {
                try!(write!(dest, "({}: ", css));
                try!(self.to_css(dest));
                write!(dest, ")")
            }
        }
    };

    (derive ToCss,FromCss for $feature:ident, $($css:expr => $variant:ident),+) => {
        impl FromCss for $feature {
            type Err = ();

            fn from_css(input: &mut Parser) -> Result<$feature, ()> {
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

    (derive EvaluateUsingContextValue<ContextValue=$context:ty> for $feature:ident) => {
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
    (derive EvaluateUsingContextValue<ContextValue=$context:ty> for $feature:ident, None = $none:expr) => {
        impl<C> EvaluateUsingContextValue<C> for $feature
            where C: DeviceFeatureContext
        {
            type ContextValue = $context;

            fn evaluate(feature_value: &Option<Self>, _: &C, context_value: $context) -> bool {
                match feature_value {
                    &Some(ref feature_value) => *feature_value == context_value,
                    &None => $none != context_value
                }
            }
        }
    };

    ($feature:ident, value: { $($css:expr => $variant:ident),+ }) => {
        discrete!(enum $feature { $($variant),+ });

        discrete!(derive to_media_feature_css for $feature);
        discrete!(derive ToCss,FromCss for $feature, $($css => $variant),+);
        discrete!(derive Parse for $feature);
        discrete!(derive EvaluateUsingContextValue<ContextValue=$feature> for $feature,
                  None = $feature::None);
    };
    ($feature:ident, value: { $($css:expr => $variant:ident),+ }, no_none) => {
        discrete!(enum $feature { $($variant),+ });

        discrete!(derive to_media_feature_css for $feature);
        discrete!(derive ToCss,FromCss for $feature, $($css => $variant),+);
        discrete!(derive Parse for $feature);
        discrete!(derive EvaluateUsingContextValue<ContextValue=$feature> for $feature);
    };

    ($feature:ident, value: mq_boolean) => {
        #[derive(Copy, Debug, PartialEq, Eq)]
        pub struct $feature(pub bool);

        discrete!(derive to_media_feature_css for $feature);
        discrete!(derive Parse for $feature);
        discrete!(derive EvaluateUsingContextValue<ContextValue=bool> for $feature,
                  None = false);

        impl PartialEq<bool> for $feature {
            #[inline]
            fn eq(&self, other: &bool) -> bool {
                self.0 == *other
            }
        }

        impl FromCss for $feature {
            type Err = ();

            fn from_css(input: &mut Parser) -> Result<$feature, ()> {
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
    };

    ($feature:ident, value: $value:ty) => {
        pub type $feature = $value;
    };
}

macro_rules! range {
    (struct $feature:ident($value:ty)) => {
        #[derive(Copy, Debug, PartialEq)]
        pub struct $feature(pub Range<$value>);
    };

    (derive to_media_feature_css for $feature:ident) => {
        impl $feature {
            fn to_media_feature_css<W>(&self, dest: &mut W, css: &'static str) -> ::text_writer::Result
                where W: ::text_writer::TextWriter
            {
                try!(write!(dest, "("));
                try!(self.0.to_css(dest, css));
                write!(dest, ")")
            }
        }
    };

    (derive Parse for $feature:ident) => {
        impl $feature {
            #[inline]
            fn parse_ident_first_form(input: &mut Parser,
                                      prefix: Option<RangePrefix>)
                                      -> Result<MediaFeature, ()>
            {
                parse_boolean_or_normal_range(input, prefix)
                    .map(|opt_range| opt_range.map(|range| $feature(range)))
                    .map(MediaFeature::$feature)
            }

            #[inline]
            fn parse_value_first_form(input: &mut Parser,
                                      after_name: SourcePosition)
                                      -> Result<MediaFeature, ()>
            {
                parse_range_form(input, after_name)
                    .map(|range| Some($feature(range)))
                    .map(MediaFeature::$feature)
            }
        }
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

    ($feature:ident,
     value: $value:ty,
     impl: {
         context: $context:ty,
         zero: $zero:expr
     }) => {
        range!(struct $feature($value));
        range!(derive to_media_feature_css for $feature);
        range!(derive Parse for $feature);
        range!(derive EvaluateUsingContextValue<ContextValue=$context> for $feature,
               Zero=$zero);
    };
    ($feature:ident,
     value: $value:ty,
     impl: {
         context: $context:ty,
         zero: $zero:expr,
         compute: $compute:ident
     }) => {
        range!(struct $feature($value));
        range!(derive to_media_feature_css for $feature);
        range!(derive Parse for $feature);
        range!(derive EvaluateUsingContextValue<ContextValue=$context> for $feature,
               Zero=$zero, Compute=$compute);
    }
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
                                               prefix: Option<RangePrefix>,
                                               name: &'a str)
                                               -> Result<MediaFeature, ()>
        {
            match name {
                $(n if $css.eq_ignore_ascii_case(n) =>
                      $feature::parse_ident_first_form(input, prefix),)+
                _ => Err(())
            }
        }

        #[inline]
        fn dispatch_parse_value_first_form<'a>(input: &mut Parser,
                                               name: &'a str,
                                               after_name: SourcePosition)
                                               -> Result<MediaFeature, ()>
        {
            match name {
                $(n if $css.eq_ignore_ascii_case(n) =>
                      $feature::parse_value_first_form(input, after_name),)+
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
            fn MediaType(&self) -> ::media_queries::query::DefinedMediaType;
            fn ViewportSize(&self) -> Size2D<Au>;

            $(fn $feature(&self) -> <$feature as EvaluateUsingContextValue<Self>>::ContextValue;)+
        }
    }
}
