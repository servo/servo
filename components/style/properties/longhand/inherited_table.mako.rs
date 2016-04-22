/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 <%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("InheritedTable", inherited=True, gecko_ffi_name="nsStyleTableBorder") %>

${helpers.single_keyword("border-collapse", "separate collapse", gecko_constant_prefix="NS_STYLE_BORDER")}
${helpers.single_keyword("empty-cells", "show hide", gecko_constant_prefix="NS_STYLE_TABLE_EMPTY_CELLS")}
${helpers.single_keyword("caption-side", "top bottom", extra_gecko_values="right left top-outside bottom-outside")}

<%helpers:longhand name="border-spacing">
    use app_units::Au;
    use values::AuExtensionMethods;

    use cssparser::ToCss;
    use std::fmt;

    pub mod computed_value {
        use app_units::Au;

        #[derive(Clone, Copy, Debug, PartialEq, RustcEncodable, HeapSizeOf)]
        pub struct T {
            pub horizontal: Au,
            pub vertical: Au,
        }
    }

    #[derive(Clone, Debug, PartialEq, HeapSizeOf)]
    pub struct SpecifiedValue {
        pub horizontal: specified::Length,
        pub vertical: specified::Length,
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T {
            horizontal: Au(0),
            vertical: Au(0),
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.horizontal.to_css(dest));
            try!(dest.write_str(" "));
            self.vertical.to_css(dest)
        }
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.horizontal.to_css(dest));
            try!(dest.write_str(" "));
            self.vertical.to_css(dest)
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
            computed_value::T {
                horizontal: self.horizontal.to_computed_value(context),
                vertical: self.vertical.to_computed_value(context),
            }
        }
    }

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        let mut lengths = [ None, None ];
        for i in 0..2 {
            match specified::Length::parse_non_negative(input) {
                Err(()) => break,
                Ok(length) => lengths[i] = Some(length),
            }
        }
        if input.next().is_ok() {
            return Err(())
        }
        match (lengths[0], lengths[1]) {
            (None, None) => Err(()),
            (Some(length), None) => {
                Ok(SpecifiedValue {
                    horizontal: length,
                    vertical: length,
                })
            }
            (Some(horizontal), Some(vertical)) => {
                Ok(SpecifiedValue {
                    horizontal: horizontal,
                    vertical: vertical,
                })
            }
            (None, Some(_)) => panic!("shouldn't happen"),
        }
    }
</%helpers:longhand>
