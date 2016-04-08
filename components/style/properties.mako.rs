/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is a Mako template: http://www.makotemplates.org/

// Please note that valid Rust syntax may be mangled by the Mako parser.
// For example, Vec<&Foo> will be mangled as Vec&Foo>. To work around these issues, the code
// can be escaped. In the above example, Vec<<&Foo> achieves the desired result of Vec<&Foo>.

use std::ascii::AsciiExt;
use std::collections::HashSet;
use std::fmt;
use std::intrinsics;
use std::mem;
use std::sync::Arc;

use app_units::Au;
use cssparser::{Parser, Color, RGBA, AtRuleParser, DeclarationParser, Delimiter,
                DeclarationListParser, parse_important, ToCss, TokenSerializationType};
use error_reporting::ParseErrorReporter;
use url::Url;
use euclid::SideOffsets2D;
use euclid::size::Size2D;
use string_cache::Atom;
use computed_values;
use logical_geometry::{LogicalMargin, PhysicalSide, WritingMode};
use parser::{ParserContext, log_css_error};
use selectors::matching::DeclarationBlock;
use stylesheets::Origin;
use values::AuExtensionMethods;
use values::computed::{self, TContext, ToComputedValue};
use values::specified::BorderStyle;

use self::property_bit_field::PropertyBitField;

<%!

import re

def to_rust_ident(name):
    name = name.replace("-", "_")
    if name in ["static", "super", "box", "move"]:  # Rust keywords
        name += "_"
    return name

def to_camel_case(ident):
    return re.sub("_([a-z])", lambda m: m.group(1).upper(), ident.strip("_").capitalize())

class Keyword(object):
    def __init__(self, name, values, extra_gecko_values=None, extra_servo_values=None):
        self.name = name
        self.values = values
        self.extra_gecko_values = extra_gecko_values or []
        self.extra_servo_values = extra_servo_values or []
    def gecko_values(self):
        return self.values + self.extra_gecko_values
    def servo_values(self):
        return self.values + self.extra_servo_values
    def values_for(self, product):
        if product == "gecko":
            return self.gecko_values()
        elif product == "servo":
            return self.servo_values()
        else:
            raise Exception("Bad product: " + product)

class Longhand(object):
    def __init__(self, name, derived_from=None, keyword=None,
                 custom_cascade=False, experimental=False, internal=False):
        self.name = name
        self.keyword = keyword
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.style_struct = THIS_STYLE_STRUCT
        self.experimental = ("layout.%s.enabled" % name) if experimental else None
        self.custom_cascade = custom_cascade
        self.internal = internal
        if derived_from is None:
            self.derived_from = None
        else:
            self.derived_from = [ to_rust_ident(name) for name in derived_from ]

class Shorthand(object):
    def __init__(self, name, sub_properties, experimental=False, internal=False):
        self.name = name
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.derived_from = None
        self.experimental = ("layout.%s.enabled" % name) if experimental else None
        self.sub_properties = [LONGHANDS_BY_NAME[s] for s in sub_properties]
        self.internal = internal

class Method(object):
    def __init__(self, name, return_type=None, arg_types=None, is_mut=False):
        self.name = name
        self.return_type = return_type
        self.arg_types = arg_types or []
        self.is_mut = is_mut
    def arg_list(self):
        args = ["_: " + x for x in self.arg_types]
        args = ["&mut self" if self.is_mut else "&self"] + args
        return ", ".join(args)
    def signature(self):
        sig = "fn %s(%s)" % (self.name, self.arg_list())
        if self.return_type:
            sig = sig + " -> " + self.return_type
        return sig
    def declare(self):
        return self.signature() + ";"
    def stub(self):
        return self.signature() + "{ unimplemented!() }"

class StyleStruct(object):
    def __init__(self, name, inherited, gecko_ffi_name, additional_methods):
        self.servo_struct_name = "Servo" + name
        self.gecko_struct_name = "Gecko" + name
        self.trait_name = name
        self.trait_name_lower = name.lower()
        self.ident = to_rust_ident(self.trait_name_lower)
        self.longhands = []
        self.inherited = inherited
        self.gecko_ffi_name = gecko_ffi_name
        self.additional_methods = additional_methods or []

STYLE_STRUCTS = []
THIS_STYLE_STRUCT = None
LONGHANDS = []
LONGHANDS_BY_NAME = {}
DERIVED_LONGHANDS = {}
SHORTHANDS = []
CONFIG = {}

def set_product(p):
    global CONFIG
    CONFIG['product'] = p

def new_style_struct(name, is_inherited, gecko_name=None, additional_methods=None):
    global THIS_STYLE_STRUCT

    style_struct = StyleStruct(name, is_inherited, gecko_name, additional_methods)
    STYLE_STRUCTS.append(style_struct)
    THIS_STYLE_STRUCT = style_struct
    return ""

def switch_to_style_struct(name):
    global THIS_STYLE_STRUCT

    for style_struct in STYLE_STRUCTS:
        if style_struct.trait_name == name:
            THIS_STYLE_STRUCT = style_struct
            return ""
    raise Exception("Failed to find the struct named " + name)
%>

// Work around Mako's really annoying namespacing setup.
//
// The above code runs when the template is loaded, rather than when it's
// rendered, so it can create global variables, doesn't have access to
// arguments passed to render(). On the flip side, there are various situations,
// such as code in the body of a def-used-as-tag, where our python code has
// access to global variables but not to render() arguments. Hack around this
// by stashing render arguments in a global.
<% CONFIG['product'] = PRODUCT %>

pub mod longhands {
    use cssparser::Parser;
    use parser::ParserContext;
    use values::specified;

    <%def name="raw_longhand(name, keyword=None, derived_from=None, products='gecko,servo',
                             custom_cascade=False, experimental=False, internal=False)">
    <%
        if not CONFIG['product'] in products:
            return ""
        if derived_from is not None:
            derived_from = derived_from.split()

        property = Longhand(name,
                            derived_from=derived_from,
                            keyword=keyword,
                            custom_cascade=custom_cascade,
                            experimental=experimental,
                            internal=internal)
        property.style_struct = THIS_STYLE_STRUCT
        THIS_STYLE_STRUCT.longhands.append(property)
        LONGHANDS.append(property)
        LONGHANDS_BY_NAME[name] = property

        if derived_from is not None:
            for name in derived_from:
                DERIVED_LONGHANDS.setdefault(name, []).append(property)
    %>
        pub mod ${property.ident} {
            #![allow(unused_imports)]
            % if derived_from is None:
                use cssparser::Parser;
                use parser::ParserContext;
                use properties::{CSSWideKeyword, DeclaredValue, Shorthand};
            % endif
            use error_reporting::ParseErrorReporter;
            use properties::longhands;
            use properties::property_bit_field::PropertyBitField;
            use properties::{ComputedValues, ServoComputedValues, PropertyDeclaration};
            use properties::style_struct_traits::T${THIS_STYLE_STRUCT.trait_name};
            use properties::style_structs;
            use std::collections::HashMap;
            use std::sync::Arc;
            use values::computed::{TContext, ToComputedValue};
            use values::{computed, specified};
            use string_cache::Atom;
            ${caller.body()}
            #[allow(unused_variables)]
            pub fn cascade_property<C: ComputedValues>(
                                    declaration: &PropertyDeclaration,
                                    inherited_style: &C,
                                    context: &mut computed::Context<C>,
                                    seen: &mut PropertyBitField,
                                    cacheable: &mut bool,
                                    error_reporter: &mut Box<ParseErrorReporter + Send>) {
                let declared_value = match *declaration {
                    PropertyDeclaration::${property.camel_case}(ref declared_value) => {
                        declared_value
                    }
                    _ => panic!("entered the wrong cascade_property() implementation"),
                };
                % if property.derived_from is None:
                    if seen.get_${property.ident}() {
                        return
                    }
                    seen.set_${property.ident}();
                    {
                        let custom_props = context.style().custom_properties();
                        ::properties::substitute_variables_${property.ident}(
                            declared_value, &custom_props, |value| match *value {
                                DeclaredValue::Value(ref specified_value) => {
                                    let computed = specified_value.to_computed_value(context);
                                    context.mutate_style().mutate_${THIS_STYLE_STRUCT.trait_name_lower}()
                                                          .set_${property.ident}(computed);
                                }
                                DeclaredValue::WithVariables { .. } => unreachable!(),
                                DeclaredValue::Initial => {
                                    // We assume that it's faster to use copy_*_from rather than
                                    // set_*(get_initial_value());
                                    let initial_struct = C::initial_values()
                                                          .get_${THIS_STYLE_STRUCT.trait_name_lower}();
                                    context.mutate_style().mutate_${THIS_STYLE_STRUCT.trait_name_lower}()
                                                          .copy_${property.ident}_from(initial_struct);
                                },
                                DeclaredValue::Inherit => {
                                    // This is a bit slow, but this is rare so it shouldn't
                                    // matter.
                                    //
                                    // FIXME: is it still?
                                    *cacheable = false;
                                    let inherited_struct = inherited_style.get_${THIS_STYLE_STRUCT.trait_name_lower}();
                                    context.mutate_style().mutate_${THIS_STYLE_STRUCT.trait_name_lower}()
                                           .copy_${property.ident}_from(inherited_struct);
                                }
                            }, error_reporter
                        );
                    }

                    % if custom_cascade:
                        cascade_property_custom(declaration,
                                                inherited_style,
                                                context,
                                                seen,
                                                cacheable,
                                                error_reporter);
                    % endif
                % else:
                    // Do not allow stylesheets to set derived properties.
                % endif
            }
            % if derived_from is None:
                pub fn parse_declared(context: &ParserContext, input: &mut Parser)
                                   -> Result<DeclaredValue<SpecifiedValue>, ()> {
                    match input.try(CSSWideKeyword::parse) {
                        Ok(CSSWideKeyword::InheritKeyword) => Ok(DeclaredValue::Inherit),
                        Ok(CSSWideKeyword::InitialKeyword) => Ok(DeclaredValue::Initial),
                        Ok(CSSWideKeyword::UnsetKeyword) => Ok(DeclaredValue::${
                            "Inherit" if THIS_STYLE_STRUCT.inherited else "Initial"}),
                        Err(()) => {
                            input.look_for_var_functions();
                            let start = input.position();
                            let specified = parse_specified(context, input);
                            if specified.is_err() {
                                while let Ok(_) = input.next() {}  // Look for var() after the error.
                            }
                            let var = input.seen_var_functions();
                            if specified.is_err() && var {
                                input.reset(start);
                                let (first_token_type, css) = try!(
                                    ::custom_properties::parse_non_custom_with_var(input));
                                return Ok(DeclaredValue::WithVariables {
                                    css: css.into_owned(),
                                    first_token_type: first_token_type,
                                    base_url: context.base_url.clone(),
                                    from_shorthand: None,
                                })
                            }
                            specified
                        }
                    }
                }
            % endif
        }
    </%def>

    <%def name="longhand(name, derived_from=None, keyword=None, products='gecko,servo',
                         custom_cascade=False, experimental=False, internal=False)">
        <%self:raw_longhand name="${name}" derived_from="${derived_from}" keyword="${keyword}"
                products="${products}" custom_cascade="${custom_cascade}"
                experimental="${experimental}" internal="${internal}">
            ${caller.body()}
            % if derived_from is None:
                pub fn parse_specified(context: &ParserContext, input: &mut Parser)
                                   -> Result<DeclaredValue<SpecifiedValue>, ()> {
                    parse(context, input).map(DeclaredValue::Value)
                }
            % endif
        </%self:raw_longhand>
    </%def>

    <%def name="single_keyword_computed(name, values, products='gecko,servo',
                                        extra_gecko_values=None, extra_servo_values=None,
                                        custom_cascade=False, experimental=False, internal=False)">
        <%self:longhand name="${name}" keyword="${Keyword(name, values.split(),
                                                          extra_gecko_values,
                                                          extra_servo_values)}"
                        products="${products}" custom_cascade="${custom_cascade}"
                        experimental="${experimental}" internal="${internal}">
            pub use self::computed_value::T as SpecifiedValue;
            ${caller.body()}
            pub mod computed_value {
                define_css_keyword_enum! { T:
                    % for value in LONGHANDS_BY_NAME[name].keyword.values_for(CONFIG['product']):
                        "${value}" => ${to_rust_ident(value)},
                    % endfor
                }
            }
            #[inline] pub fn get_initial_value() -> computed_value::T {
                computed_value::T::${to_rust_ident(values.split()[0])}
            }
            pub fn parse(_context: &ParserContext, input: &mut Parser)
                         -> Result<SpecifiedValue, ()> {
                computed_value::T::parse(input)
            }
        </%self:longhand>
    </%def>

    <%def name="single_keyword(name, values, products='gecko,servo', experimental=False, internal=False)">
        <%self:single_keyword_computed name="${name}"
                                       values="${values}"
                                       products="${products}"
                                       experimental="${experimental}"
                                       internal="${internal}">
            use values::computed::ComputedValueAsSpecified;
            impl ComputedValueAsSpecified for SpecifiedValue {}
        </%self:single_keyword_computed>
    </%def>

    <%def name="predefined_type(name, type, initial_value, parse_method='parse', products='gecko,servo')">
        <%self:longhand name="${name}" products="${products}">
            #[allow(unused_imports)]
            use app_units::Au;
            pub type SpecifiedValue = specified::${type};
            pub mod computed_value {
                pub use values::computed::${type} as T;
            }
            #[inline] pub fn get_initial_value() -> computed_value::T { ${initial_value} }
            #[inline] pub fn parse(_context: &ParserContext, input: &mut Parser)
                                   -> Result<SpecifiedValue, ()> {
                specified::${type}::${parse_method}(input)
            }
        </%self:longhand>
    </%def>


    // CSS 2.1, Section 8 - Box model

    ${new_style_struct("Margin", is_inherited=False, gecko_name="nsStyleMargin")}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("margin-" + side, "LengthOrPercentageOrAuto",
                          "computed::LengthOrPercentageOrAuto::Length(Au(0))")}
    % endfor

    ${new_style_struct("Padding", is_inherited=False, gecko_name="nsStylePadding")}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("padding-" + side, "LengthOrPercentage",
                          "computed::LengthOrPercentage::Length(Au(0))",
                          "parse_non_negative")}
    % endfor

    ${new_style_struct("Border", is_inherited=False, gecko_name="nsStyleBorder",
                       additional_methods=[Method("border_" + side + "_is_none_or_hidden_and_has_nonzero_width",
                                                  "bool") for side in ["top", "right", "bottom", "left"]])}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("border-%s-color" % side, "CSSColor", "::cssparser::Color::CurrentColor")}
    % endfor

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("border-%s-style" % side, "BorderStyle", "specified::BorderStyle::none")}
    % endfor

    % for side in ["top", "right", "bottom", "left"]:
        <%self:longhand name="border-${side}-width">
            use app_units::Au;
            use cssparser::ToCss;
            use std::fmt;

            impl ToCss for SpecifiedValue {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    self.0.to_css(dest)
                }
            }

            #[inline]
            pub fn parse(_context: &ParserContext, input: &mut Parser)
                                   -> Result<SpecifiedValue, ()> {
                specified::parse_border_width(input).map(SpecifiedValue)
            }
            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct SpecifiedValue(pub specified::Length);
            pub mod computed_value {
                use app_units::Au;
                pub type T = Au;
            }
            #[inline] pub fn get_initial_value() -> computed_value::T {
                Au::from_px(3)  // medium
            }

            impl ToComputedValue for SpecifiedValue {
                type ComputedValue = computed_value::T;

                #[inline]
                fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                    self.0.to_computed_value(context)
                }
            }
        </%self:longhand>
    % endfor

    // FIXME(#4126): when gfx supports painting it, make this Size2D<LengthOrPercentage>
    % for corner in ["top-left", "top-right", "bottom-right", "bottom-left"]:
        ${predefined_type("border-" + corner + "-radius", "BorderRadiusSize",
                          "computed::BorderRadiusSize::zero()",
                          "parse")}
    % endfor

    ${new_style_struct("Outline", is_inherited=False, gecko_name="nsStyleOutline",
                       additional_methods=[Method("outline_is_none_or_hidden_and_has_nonzero_width", "bool")])}

    // TODO(pcwalton): `invert`
    ${predefined_type("outline-color", "CSSColor", "::cssparser::Color::CurrentColor")}

    <%self:longhand name="outline-style">
        pub use values::specified::BorderStyle as SpecifiedValue;
        pub fn get_initial_value() -> SpecifiedValue { SpecifiedValue::none }
        pub mod computed_value {
            pub use values::specified::BorderStyle as T;
        }
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            match SpecifiedValue::parse(input) {
                Ok(SpecifiedValue::hidden) => Err(()),
                result => result
            }
        }
    </%self:longhand>

    <%self:longhand name="outline-width">
        use app_units::Au;
        use cssparser::ToCss;
        use std::fmt;
        use values::AuExtensionMethods;

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.0.to_css(dest)
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            specified::parse_border_width(input).map(SpecifiedValue)
        }
        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct SpecifiedValue(pub specified::Length);
        pub mod computed_value {
            use app_units::Au;
            pub type T = Au;
        }
        pub use super::border_top_width::get_initial_value;
        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                self.0.to_computed_value(context)
            }
        }
    </%self:longhand>

    ${predefined_type("outline-offset", "Length", "Au(0)")}

    ${new_style_struct("Position", is_inherited=False, gecko_name="nsStylePosition")}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type(side, "LengthOrPercentageOrAuto",
                          "computed::LengthOrPercentageOrAuto::Auto")}
    % endfor

    // CSS 2.1, Section 9 - Visual formatting model

    ${new_style_struct("Box", is_inherited=False, gecko_name="nsStyleDisplay",
                       additional_methods=[Method("clone_display",
                                                  "longhands::display::computed_value::T"),
                                           Method("clone_position",
                                                  "longhands::position::computed_value::T"),
                                           Method("is_floated", "bool"),
                                           Method("overflow_x_is_visible", "bool"),
                                           Method("overflow_y_is_visible", "bool"),
                                           Method("transition_count", "usize")])}

    // TODO(SimonSapin): don't parse `inline-table`, since we don't support it
    <%self:longhand name="display" custom_cascade="True">
        <%
            values = """inline block inline-block
                table inline-table table-row-group table-header-group table-footer-group
                table-row table-column-group table-column table-cell table-caption
                list-item flex
                none
            """.split()
            experimental_values = set("flex".split())
        %>
        pub use self::computed_value::T as SpecifiedValue;
        use values::computed::{Context, ComputedValueAsSpecified};
        use properties::style_struct_traits::TInheritedText;

        pub mod computed_value {
            #[allow(non_camel_case_types)]
            #[derive(Clone, Eq, PartialEq, Copy, Hash, RustcEncodable, Debug, HeapSizeOf)]
            #[derive(Deserialize, Serialize)]
            pub enum T {
                % for value in values:
                    ${to_rust_ident(value)},
                % endfor
            }

            impl ::cssparser::ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
                where W: ::std::fmt::Write {
                    match *self {
                        % for value in values:
                            T::${to_rust_ident(value)} => dest.write_str("${value}"),
                        % endfor
                    }
                }
            }
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            computed_value::T::${to_rust_ident(values[0])}
        }
        pub fn parse(_context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            match_ignore_ascii_case! { try!(input.expect_ident()),
                % for value in values:
                    "${value}" => {
                        % if value in experimental_values:
                            if !::util::prefs::get_pref("layout.${value}.enabled")
                                .as_boolean().unwrap_or(false) {
                                return Err(())
                            }
                        % endif
                        Ok(computed_value::T::${to_rust_ident(value)})
                    },
                % endfor
                _ => Err(())
            }
        }

        impl ComputedValueAsSpecified for SpecifiedValue {}

        fn cascade_property_custom<C: ComputedValues>(
                                   _declaration: &PropertyDeclaration,
                                   _inherited_style: &C,
                                   context: &mut computed::Context<C>,
                                   _seen: &mut PropertyBitField,
                                   _cacheable: &mut bool,
                                   _error_reporter: &mut Box<ParseErrorReporter + Send>) {
            longhands::_servo_display_for_hypothetical_box::derive_from_display(context);
            longhands::_servo_text_decorations_in_effect::derive_from_display(context);
        }
    </%self:longhand>

    ${single_keyword("position", "static absolute relative fixed")}

    <%self:single_keyword_computed name="float" values="none left right">
        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                let positioned = matches!(context.style().get_box().clone_position(),
                    longhands::position::SpecifiedValue::absolute |
                    longhands::position::SpecifiedValue::fixed);
                if positioned {
                    SpecifiedValue::none
                } else {
                    *self
                }
            }
        }

    </%self:single_keyword_computed>

    ${single_keyword("clear", "none left right both")}

    <%self:longhand name="-servo-display-for-hypothetical-box" derived_from="display">
        pub use super::display::{SpecifiedValue, get_initial_value};
        pub use super::display::{parse};

        pub mod computed_value {
            pub type T = super::SpecifiedValue;
        }

        #[inline]
        pub fn derive_from_display<Cx: TContext>(context: &mut Cx) {
            let d = context.style().get_box().clone_display();
            context.mutate_style().mutate_box().set__servo_display_for_hypothetical_box(d);
        }

    </%self:longhand>

    ${switch_to_style_struct("Position")}

    <%self:longhand name="z-index">
        use values::computed::ComputedValueAsSpecified;

        impl ComputedValueAsSpecified for SpecifiedValue {}
        pub type SpecifiedValue = computed_value::T;
        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;

            #[derive(PartialEq, Clone, Eq, Copy, Debug, HeapSizeOf)]
            pub enum T {
                Auto,
                Number(i32),
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        T::Auto => dest.write_str("auto"),
                        T::Number(number) => write!(dest, "{}", number),
                    }
                }
            }

            impl T {
                pub fn number_or_zero(self) -> i32 {
                    match self {
                        T::Auto => 0,
                        T::Number(value) => value,
                    }
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::Auto
        }
        fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                Ok(computed_value::T::Auto)
            } else {
                specified::parse_integer(input).map(computed_value::T::Number)
            }
        }
    </%self:longhand>

    ${new_style_struct("InheritedBox", is_inherited=True,
                       additional_methods=[Method("clone_direction",
                                                  "longhands::direction::computed_value::T"),
                                           Method("clone_writing_mode",
                                                  "longhands::writing_mode::computed_value::T"),
                                           Method("clone_text_orientation",
                                                  "longhands::text_orientation::computed_value::T")])}

    ${single_keyword("direction", "ltr rtl")}

    // CSS 2.1, Section 10 - Visual formatting model details

    ${switch_to_style_struct("Box")}

    ${predefined_type("width", "LengthOrPercentageOrAuto",
                      "computed::LengthOrPercentageOrAuto::Auto",
                      "parse_non_negative")}

    ${predefined_type("height", "LengthOrPercentageOrAuto",
                      "computed::LengthOrPercentageOrAuto::Auto",
                      "parse_non_negative")}

    ${switch_to_style_struct("Position")}

    ${predefined_type("min-width", "LengthOrPercentage",
                      "computed::LengthOrPercentage::Length(Au(0))",
                      "parse_non_negative")}
    ${predefined_type("max-width", "LengthOrPercentageOrNone",
                      "computed::LengthOrPercentageOrNone::None",
                      "parse_non_negative")}

    ${predefined_type("min-height", "LengthOrPercentage",
                      "computed::LengthOrPercentage::Length(Au(0))",
                      "parse_non_negative")}
    ${predefined_type("max-height", "LengthOrPercentageOrNone",
                      "computed::LengthOrPercentageOrNone::None",
                      "parse_non_negative")}

    ${new_style_struct("InheritedText", is_inherited=True, gecko_name="nsStyleText",
                       additional_methods=[Method("clone__servo_text_decorations_in_effect",
                                                  "longhands::_servo_text_decorations_in_effect::computed_value::T")])}

    <%self:longhand name="line-height">
        use cssparser::ToCss;
        use std::fmt;
        use values::AuExtensionMethods;
        use values::CSSFloat;

        #[derive(Debug, Clone, PartialEq, Copy, HeapSizeOf)]
        pub enum SpecifiedValue {
            Normal,
            Number(CSSFloat),
            LengthOrPercentage(specified::LengthOrPercentage),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Normal => dest.write_str("normal"),
                    SpecifiedValue::LengthOrPercentage(value) => value.to_css(dest),
                    SpecifiedValue::Number(number) => write!(dest, "{}", number),
                }
            }
        }
        /// normal | <number> | <length> | <percentage>
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            use cssparser::Token;
            use std::ascii::AsciiExt;
            input.try(specified::LengthOrPercentage::parse_non_negative)
            .map(SpecifiedValue::LengthOrPercentage)
            .or_else(|()| {
                match try!(input.next()) {
                    Token::Number(ref value) if value.value >= 0. => {
                        Ok(SpecifiedValue::Number(value.value))
                    }
                    Token::Ident(ref value) if value.eq_ignore_ascii_case("normal") => {
                        Ok(SpecifiedValue::Normal)
                    }
                    _ => Err(()),
                }
            })
        }
        pub mod computed_value {
            use app_units::Au;
            use std::fmt;
            use values::CSSFloat;
            #[derive(PartialEq, Copy, Clone, HeapSizeOf, Debug)]
            pub enum T {
                Normal,
                Length(Au),
                Number(CSSFloat),
            }
        }
        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    computed_value::T::Normal => dest.write_str("normal"),
                    computed_value::T::Length(length) => length.to_css(dest),
                    computed_value::T::Number(number) => write!(dest, "{}", number),
                }
            }
        }
         #[inline]
        pub fn get_initial_value() -> computed_value::T { computed_value::T::Normal }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                match *self {
                    SpecifiedValue::Normal => computed_value::T::Normal,
                    SpecifiedValue::Number(value) => computed_value::T::Number(value),
                    SpecifiedValue::LengthOrPercentage(value) => {
                        match value {
                            specified::LengthOrPercentage::Length(value) =>
                                computed_value::T::Length(value.to_computed_value(context)),
                            specified::LengthOrPercentage::Percentage(specified::Percentage(value)) => {
                                let fr = specified::Length::FontRelative(specified::FontRelativeLength::Em(value));
                                computed_value::T::Length(fr.to_computed_value(context))
                            },
                            specified::LengthOrPercentage::Calc(calc) => {
                                let calc = calc.to_computed_value(context);
                                let fr = specified::FontRelativeLength::Em(calc.percentage());
                                let fr = specified::Length::FontRelative(fr);
                                computed_value::T::Length(calc.length() + fr.to_computed_value(context))
                            }
                        }
                    }
                }
            }
        }
    </%self:longhand>

    ${switch_to_style_struct("Box")}

    <%self:longhand name="vertical-align">
        use cssparser::ToCss;
        use std::fmt;

        <% vertical_align_keywords = (
            "baseline sub super top text-top middle bottom text-bottom".split()) %>
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, PartialEq, Copy, HeapSizeOf)]
        pub enum SpecifiedValue {
            % for keyword in vertical_align_keywords:
                ${to_rust_ident(keyword)},
            % endfor
            LengthOrPercentage(specified::LengthOrPercentage),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    % for keyword in vertical_align_keywords:
                        SpecifiedValue::${to_rust_ident(keyword)} => dest.write_str("${keyword}"),
                    % endfor
                    SpecifiedValue::LengthOrPercentage(value) => value.to_css(dest),
                }
            }
        }
        /// baseline | sub | super | top | text-top | middle | bottom | text-bottom
        /// | <percentage> | <length>
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            input.try(specified::LengthOrPercentage::parse)
            .map(SpecifiedValue::LengthOrPercentage)
            .or_else(|()| {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    % for keyword in vertical_align_keywords:
                        "${keyword}" => Ok(SpecifiedValue::${to_rust_ident(keyword)}),
                    % endfor
                    _ => Err(())
                }
            })
        }
        pub mod computed_value {
            use app_units::Au;
            use std::fmt;
            use values::AuExtensionMethods;
            use values::{CSSFloat, computed};
            #[allow(non_camel_case_types)]
            #[derive(PartialEq, Copy, Clone, HeapSizeOf, Debug)]
            pub enum T {
                % for keyword in vertical_align_keywords:
                    ${to_rust_ident(keyword)},
                % endfor
                LengthOrPercentage(computed::LengthOrPercentage),
            }
            impl ::cssparser::ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        % for keyword in vertical_align_keywords:
                            T::${to_rust_ident(keyword)} => dest.write_str("${keyword}"),
                        % endfor
                        T::LengthOrPercentage(value) => value.to_css(dest),
                    }
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T { computed_value::T::baseline }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                match *self {
                    % for keyword in vertical_align_keywords:
                        SpecifiedValue::${to_rust_ident(keyword)} => {
                            computed_value::T::${to_rust_ident(keyword)}
                        }
                    % endfor
                    SpecifiedValue::LengthOrPercentage(value) =>
                        computed_value::T::LengthOrPercentage(value.to_computed_value(context)),
                }
            }
        }
    </%self:longhand>


    // CSS 2.1, Section 11 - Visual effects

    // Non-standard, see https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box#Specifications
    ${single_keyword("-servo-overflow-clip-box", "padding-box content-box", internal=True)}

    // FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
    ${single_keyword("overflow-x", "visible hidden scroll auto")}

    // FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
    <%self:longhand name="overflow-y">
        use super::overflow_x;

        use cssparser::ToCss;
        use std::fmt;

        pub use self::computed_value::T as SpecifiedValue;

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.0.to_css(dest)
            }
        }

        pub mod computed_value {
            #[derive(Debug, Clone, Copy, PartialEq, HeapSizeOf)]
            pub struct T(pub super::super::overflow_x::computed_value::T);
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                computed_value::T(self.0.to_computed_value(context))
            }
        }

        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(overflow_x::get_initial_value())
        }

        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            overflow_x::parse(context, input).map(SpecifiedValue)
        }
    </%self:longhand>

    ${switch_to_style_struct("InheritedBox")}

    // TODO: collapse. Well, do tables first.
    ${single_keyword("visibility", "visible hidden")}

    // CSS 2.1, Section 12 - Generated content, automatic numbering, and lists

    ${new_style_struct("Counters", is_inherited=False)}

    <%self:longhand name="content">
        use cssparser::Token;
        use std::ascii::AsciiExt;
        use values::computed::ComputedValueAsSpecified;

        use super::list_style_type;

        pub use self::computed_value::T as SpecifiedValue;
        pub use self::computed_value::ContentItem;

        impl ComputedValueAsSpecified for SpecifiedValue {}

        pub mod computed_value {
            use super::super::list_style_type;

            use cssparser::{self, ToCss};
            use std::fmt;

            #[derive(Debug, PartialEq, Eq, Clone, HeapSizeOf)]
            pub enum ContentItem {
                /// Literal string content.
                String(String),
                /// `counter(name, style)`.
                Counter(String, list_style_type::computed_value::T),
                /// `counters(name, separator, style)`.
                Counters(String, String, list_style_type::computed_value::T),
                /// `open-quote`.
                OpenQuote,
                /// `close-quote`.
                CloseQuote,
                /// `no-open-quote`.
                NoOpenQuote,
                /// `no-close-quote`.
                NoCloseQuote,
            }

            impl ToCss for ContentItem {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        ContentItem::String(ref s) => {
                            cssparser::serialize_string(&**s, dest)
                        }
                        ContentItem::Counter(ref s, ref list_style_type) => {
                            try!(dest.write_str("counter("));
                            try!(cssparser::serialize_identifier(&**s, dest));
                            try!(dest.write_str(", "));
                            try!(list_style_type.to_css(dest));
                            dest.write_str(")")
                        }
                        ContentItem::Counters(ref s, ref separator, ref list_style_type) => {
                            try!(dest.write_str("counter("));
                            try!(cssparser::serialize_identifier(&**s, dest));
                            try!(dest.write_str(", "));
                            try!(cssparser::serialize_string(&**separator, dest));
                            try!(dest.write_str(", "));
                            try!(list_style_type.to_css(dest));
                            dest.write_str(")")
                        }
                        ContentItem::OpenQuote => dest.write_str("open-quote"),
                        ContentItem::CloseQuote => dest.write_str("close-quote"),
                        ContentItem::NoOpenQuote => dest.write_str("no-open-quote"),
                        ContentItem::NoCloseQuote => dest.write_str("no-close-quote"),
                    }
                }
            }

            #[allow(non_camel_case_types)]
            #[derive(Debug, PartialEq, Eq, Clone, HeapSizeOf)]
            pub enum T {
                normal,
                none,
                Content(Vec<ContentItem>),
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        T::normal => dest.write_str("normal"),
                        T::none => dest.write_str("none"),
                        T::Content(ref content) => {
                            let mut iter = content.iter();
                            try!(iter.next().unwrap().to_css(dest));
                            for c in iter {
                                try!(dest.write_str(" "));
                                try!(c.to_css(dest));
                            }
                            Ok(())
                        }
                    }
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T  {
            computed_value::T::normal
        }

        pub fn counter_name_is_illegal(name: &str) -> bool {
            name.eq_ignore_ascii_case("none") || name.eq_ignore_ascii_case("inherit") ||
                name.eq_ignore_ascii_case("initial")
        }

        // normal | none | [ <string> | <counter> | open-quote | close-quote | no-open-quote |
        // no-close-quote ]+
        // TODO: <uri>, attr(<identifier>)
        pub fn parse(context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                return Ok(SpecifiedValue::normal)
            }
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue::none)
            }
            let mut content = vec![];
            loop {
                match input.next() {
                    Ok(Token::QuotedString(value)) => {
                        content.push(ContentItem::String(value.into_owned()))
                    }
                    Ok(Token::Function(name)) => {
                        content.push(try!(match_ignore_ascii_case! { name,
                            "counter" => input.parse_nested_block(|input| {
                                let name = try!(input.expect_ident()).into_owned();
                                let style = input.try(|input| {
                                    try!(input.expect_comma());
                                    list_style_type::parse(context, input)
                                }).unwrap_or(list_style_type::computed_value::T::decimal);
                                Ok(ContentItem::Counter(name, style))
                            }),
                            "counters" => input.parse_nested_block(|input| {
                                let name = try!(input.expect_ident()).into_owned();
                                try!(input.expect_comma());
                                let separator = try!(input.expect_string()).into_owned();
                                let style = input.try(|input| {
                                    try!(input.expect_comma());
                                    list_style_type::parse(context, input)
                                }).unwrap_or(list_style_type::computed_value::T::decimal);
                                Ok(ContentItem::Counters(name, separator, style))
                            }),
                            _ => return Err(())
                        }));
                    }
                    Ok(Token::Ident(ident)) => {
                        match_ignore_ascii_case! { ident,
                            "open-quote" => content.push(ContentItem::OpenQuote),
                            "close-quote" => content.push(ContentItem::CloseQuote),
                            "no-open-quote" => content.push(ContentItem::NoOpenQuote),
                            "no-close-quote" => content.push(ContentItem::NoCloseQuote),
                            _ => return Err(())
                        }
                    }
                    Err(_) => break,
                    _ => return Err(())
                }
            }
            if !content.is_empty() {
                Ok(SpecifiedValue::Content(content))
            } else {
                Err(())
            }
        }
    </%self:longhand>

    ${new_style_struct("List", is_inherited=True, gecko_name="nsStyleList")}

    ${single_keyword("list-style-position", "outside inside")}

    // TODO(pcwalton): Implement the full set of counter styles per CSS-COUNTER-STYLES [1] 6.1:
    //
    //     decimal-leading-zero, armenian, upper-armenian, lower-armenian, georgian, lower-roman,
    //     upper-roman
    //
    // [1]: http://dev.w3.org/csswg/css-counter-styles/
    ${single_keyword("list-style-type", """
        disc none circle square decimal arabic-indic bengali cambodian cjk-decimal devanagari
        gujarati gurmukhi kannada khmer lao malayalam mongolian myanmar oriya persian telugu thai
        tibetan lower-alpha upper-alpha cjk-earthly-branch cjk-heavenly-stem lower-greek hiragana
        hiragana-iroha katakana katakana-iroha disclosure-open disclosure-closed
    """)}

    <%self:longhand name="list-style-image">
        use cssparser::{ToCss, Token};
        use std::fmt;
        use url::Url;
        use values::LocalToCss;

        #[derive(Debug, Clone, PartialEq, Eq, HeapSizeOf)]
        pub enum SpecifiedValue {
            None,
            Url(Url),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::None => dest.write_str("none"),
                    SpecifiedValue::Url(ref url) => url.to_css(dest),
                }
            }
        }

        pub mod computed_value {
            use cssparser::{ToCss, Token};
            use std::fmt;
            use url::Url;
            use values::LocalToCss;

            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Url>);

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match self.0 {
                        None => dest.write_str("none"),
                        Some(ref url) => url.to_css(dest),
                    }
                }
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, _context: &Cx) -> computed_value::T {
                match *self {
                    SpecifiedValue::None => computed_value::T(None),
                    SpecifiedValue::Url(ref url) => computed_value::T(Some(url.clone())),
                }
            }
        }

        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                Ok(SpecifiedValue::None)
            } else {
                Ok(SpecifiedValue::Url(context.parse_url(&*try!(input.expect_url()))))
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }
    </%self:longhand>

    <%self:longhand name="quotes">
        use std::borrow::Cow;
        use std::fmt;
        use values::computed::ComputedValueAsSpecified;

        use cssparser::{ToCss, Token};

        pub use self::computed_value::T as SpecifiedValue;

        pub mod computed_value {
            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Vec<(String,String)>);
        }

        impl ComputedValueAsSpecified for SpecifiedValue {}

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut first = true;
                for pair in &self.0 {
                    if !first {
                        try!(dest.write_str(" "));
                    }
                    first = false;
                    try!(Token::QuotedString(Cow::from(&*pair.0)).to_css(dest));
                    try!(dest.write_str(" "));
                    try!(Token::QuotedString(Cow::from(&*pair.1)).to_css(dest));
                }
                Ok(())
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(vec![
                ("\u{201c}".to_owned(), "\u{201d}".to_owned()),
                ("\u{2018}".to_owned(), "\u{2019}".to_owned()),
            ])
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue(Vec::new()))
            }

            let mut quotes = Vec::new();
            loop {
                let first = match input.next() {
                    Ok(Token::QuotedString(value)) => value.into_owned(),
                    Ok(_) => return Err(()),
                    Err(()) => break,
                };
                let second = match input.next() {
                    Ok(Token::QuotedString(value)) => value.into_owned(),
                    _ => return Err(()),
                };
                quotes.push((first, second))
            }
            if !quotes.is_empty() {
                Ok(SpecifiedValue(quotes))
            } else {
                Err(())
            }
        }
    </%self:longhand>

    ${switch_to_style_struct("Counters")}

    <%self:longhand name="counter-increment">
        use std::fmt;
        use super::content;
        use values::computed::ComputedValueAsSpecified;

        use cssparser::{ToCss, Token, serialize_identifier};
        use std::borrow::{Cow, ToOwned};

        pub use self::computed_value::T as SpecifiedValue;

        pub mod computed_value {
            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Vec<(String,i32)>);
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(Vec::new())
        }

        impl ComputedValueAsSpecified for SpecifiedValue {}

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut first = true;
                for pair in &self.0 {
                    if !first {
                        try!(dest.write_str(" "));
                    }
                    first = false;
                    try!(serialize_identifier(&pair.0, dest));
                    try!(write!(dest, " {}", pair.1));
                }
                Ok(())
            }
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            parse_common(1, input)
        }

        pub fn parse_common(default_value: i32, input: &mut Parser) -> Result<SpecifiedValue,()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue(Vec::new()))
            }

            let mut counters = Vec::new();
            loop {
                let counter_name = match input.next() {
                    Ok(Token::Ident(ident)) => (*ident).to_owned(),
                    Ok(_) => return Err(()),
                    Err(_) => break,
                };
                if content::counter_name_is_illegal(&counter_name) {
                    return Err(())
                }
                let counter_delta =
                    input.try(|input| specified::parse_integer(input)).unwrap_or(default_value);
                counters.push((counter_name, counter_delta))
            }

            if !counters.is_empty() {
                Ok(SpecifiedValue(counters))
            } else {
                Err(())
            }
        }
    </%self:longhand>

    <%self:longhand name="counter-reset">
        pub use super::counter_increment::{SpecifiedValue, computed_value, get_initial_value};
        use super::counter_increment::{parse_common};

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            parse_common(0, input)
        }
    </%self:longhand>

    // CSS 2.1, Section 13 - Paged media

    // CSS 2.1, Section 14 - Colors and Backgrounds

    ${new_style_struct("Background", is_inherited=False, gecko_name="nsStyleBackground")}
    ${predefined_type(
        "background-color", "CSSColor",
        "::cssparser::Color::RGBA(::cssparser::RGBA { red: 0., green: 0., blue: 0., alpha: 0. }) /* transparent */")}

    <%self:longhand name="background-image">
        use cssparser::ToCss;
        use std::fmt;
        use values::specified::Image;
        use values::LocalToCss;

        pub mod computed_value {
            use values::computed;
            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<computed::Image>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("none"),
                    Some(computed::Image::Url(ref url)) => url.to_css(dest),
                    Some(computed::Image::LinearGradient(ref gradient)) =>
                        gradient.to_css(dest)
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct SpecifiedValue(pub Option<Image>);

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue(Some(ref image)) => image.to_css(dest),
                    SpecifiedValue(None) => dest.write_str("none"),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }
        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                Ok(SpecifiedValue(None))
            } else {
                Ok(SpecifiedValue(Some(try!(Image::parse(context, input)))))
            }
        }
        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                match *self {
                    SpecifiedValue(None) => computed_value::T(None),
                    SpecifiedValue(Some(ref image)) =>
                        computed_value::T(Some(image.to_computed_value(context))),
                }
            }
        }
    </%self:longhand>

    <%self:longhand name="background-position">
            use cssparser::ToCss;
            use std::fmt;
            use values::AuExtensionMethods;

            pub mod computed_value {
                use values::computed::LengthOrPercentage;

                #[derive(PartialEq, Copy, Clone, Debug, HeapSizeOf)]
                pub struct T {
                    pub horizontal: LengthOrPercentage,
                    pub vertical: LengthOrPercentage,
                }
            }

            #[derive(Debug, Clone, PartialEq, Copy, HeapSizeOf)]
            pub struct SpecifiedValue {
                pub horizontal: specified::LengthOrPercentage,
                pub vertical: specified::LengthOrPercentage,
            }

            impl ToCss for SpecifiedValue {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    try!(self.horizontal.to_css(dest));
                    try!(dest.write_str(" "));
                    try!(self.vertical.to_css(dest));
                    Ok(())
                }
            }

            impl ToCss for computed_value::T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    try!(self.horizontal.to_css(dest));
                    try!(dest.write_str(" "));
                    try!(self.vertical.to_css(dest));
                    Ok(())
                }
            }

            impl SpecifiedValue {
                fn new(first: specified::PositionComponent, second: specified::PositionComponent)
                        -> Result<SpecifiedValue, ()> {
                    let (horiz, vert) = match (category(first), category(second)) {
                        // Don't allow two vertical keywords or two horizontal keywords.
                        (PositionCategory::HorizontalKeyword, PositionCategory::HorizontalKeyword) |
                        (PositionCategory::VerticalKeyword, PositionCategory::VerticalKeyword) => return Err(()),

                        // Swap if both are keywords and vertical precedes horizontal.
                        (PositionCategory::VerticalKeyword, PositionCategory::HorizontalKeyword) |
                        (PositionCategory::VerticalKeyword, PositionCategory::OtherKeyword) |
                        (PositionCategory::OtherKeyword, PositionCategory::HorizontalKeyword) => (second, first),

                        // By default, horizontal is first.
                        _ => (first, second),
                    };
                    Ok(SpecifiedValue {
                        horizontal: horiz.to_length_or_percentage(),
                        vertical: vert.to_length_or_percentage(),
                    })
                }
            }

            // Collapse `Position` into a few categories to simplify the above `match` expression.
            enum PositionCategory {
                HorizontalKeyword,
                VerticalKeyword,
                OtherKeyword,
                LengthOrPercentage,
            }
            fn category(p: specified::PositionComponent) -> PositionCategory {
                match p {
                    specified::PositionComponent::Left |
                    specified::PositionComponent::Right =>
                        PositionCategory::HorizontalKeyword,
                    specified::PositionComponent::Top |
                    specified::PositionComponent::Bottom =>
                        PositionCategory::VerticalKeyword,
                    specified::PositionComponent::Center =>
                        PositionCategory::OtherKeyword,
                    specified::PositionComponent::LengthOrPercentage(_) =>
                        PositionCategory::LengthOrPercentage,
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

            #[inline]
            pub fn get_initial_value() -> computed_value::T {
                computed_value::T {
                    horizontal: computed::LengthOrPercentage::Percentage(0.0),
                    vertical: computed::LengthOrPercentage::Percentage(0.0),
                }
            }

            pub fn parse(_context: &ParserContext, input: &mut Parser)
                         -> Result<SpecifiedValue, ()> {
                let first = try!(specified::PositionComponent::parse(input));
                let second = input.try(specified::PositionComponent::parse)
                    .unwrap_or(specified::PositionComponent::Center);
                SpecifiedValue::new(first, second)
            }
    </%self:longhand>

    ${single_keyword("background-repeat", "repeat repeat-x repeat-y no-repeat")}

    ${single_keyword("background-attachment", "scroll fixed")}

    ${single_keyword("background-clip", "border-box padding-box content-box")}

    ${single_keyword("background-origin", "padding-box border-box content-box")}

    <%self:longhand name="background-size">
        use cssparser::{ToCss, Token};
        use std::ascii::AsciiExt;
        use std::fmt;

        pub mod computed_value {
            use values::computed::LengthOrPercentageOrAuto;

            #[derive(PartialEq, Clone, Debug, HeapSizeOf)]
            pub struct ExplicitSize {
                pub width: LengthOrPercentageOrAuto,
                pub height: LengthOrPercentageOrAuto,
            }

            #[derive(PartialEq, Clone, Debug, HeapSizeOf)]
            pub enum T {
                Explicit(ExplicitSize),
                Cover,
                Contain,
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    computed_value::T::Explicit(ref size) => size.to_css(dest),
                    computed_value::T::Cover => dest.write_str("cover"),
                    computed_value::T::Contain => dest.write_str("contain"),
                }
            }
        }

        #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
        pub struct SpecifiedExplicitSize {
            pub width: specified::LengthOrPercentageOrAuto,
            pub height: specified::LengthOrPercentageOrAuto,
        }

        impl ToCss for SpecifiedExplicitSize {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.width.to_css(dest));
                try!(dest.write_str(" "));
                self.height.to_css(dest)
            }
        }

        impl ToCss for computed_value::ExplicitSize {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.width.to_css(dest));
                try!(dest.write_str(" "));
                self.height.to_css(dest)
            }
        }


        #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
        pub enum SpecifiedValue {
            Explicit(SpecifiedExplicitSize),
            Cover,
            Contain,
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Explicit(ref size) => size.to_css(dest),
                    SpecifiedValue::Cover => dest.write_str("cover"),
                    SpecifiedValue::Contain => dest.write_str("contain"),
                }
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                match *self {
                    SpecifiedValue::Explicit(ref size) => {
                        computed_value::T::Explicit(computed_value::ExplicitSize {
                            width: size.width.to_computed_value(context),
                            height: size.height.to_computed_value(context),
                        })
                    }
                    SpecifiedValue::Cover => computed_value::T::Cover,
                    SpecifiedValue::Contain => computed_value::T::Contain,
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::Explicit(computed_value::ExplicitSize {
                width: computed::LengthOrPercentageOrAuto::Auto,
                height: computed::LengthOrPercentageOrAuto::Auto,
            })
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            let width;
            if let Ok(value) = input.try(|input| {
                match input.next() {
                    Err(_) => Err(()),
                    Ok(Token::Ident(ref ident)) if ident.eq_ignore_ascii_case("cover") => {
                        Ok(SpecifiedValue::Cover)
                    }
                    Ok(Token::Ident(ref ident)) if ident.eq_ignore_ascii_case("contain") => {
                        Ok(SpecifiedValue::Contain)
                    }
                    Ok(_) => Err(()),
                }
            }) {
                return Ok(value)
            } else {
                width = try!(specified::LengthOrPercentageOrAuto::parse(input))
            }

            let height;
            if let Ok(value) = input.try(|input| {
                match input.next() {
                    Err(_) => Ok(specified::LengthOrPercentageOrAuto::Auto),
                    Ok(_) => Err(()),
                }
            }) {
                height = value
            } else {
                height = try!(specified::LengthOrPercentageOrAuto::parse(input));
            }

            Ok(SpecifiedValue::Explicit(SpecifiedExplicitSize {
                width: width,
                height: height,
            }))
        }
    </%self:longhand>

    ${new_style_struct("Color", is_inherited=True, gecko_name="nsStyleColor",
                       additional_methods=[Method("clone_color",
                                                  "longhands::color::computed_value::T")])}

    <%self:raw_longhand name="color">
        use cssparser::{Color, RGBA};
        use values::specified::{CSSColor, CSSRGBA};

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, _context: &Cx) -> computed_value::T {
                self.parsed
            }
        }

        pub type SpecifiedValue = CSSRGBA;
        pub mod computed_value {
            use cssparser;
            pub type T = cssparser::RGBA;
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            RGBA { red: 0., green: 0., blue: 0., alpha: 1. }  /* black */
        }
        pub fn parse_specified(_context: &ParserContext, input: &mut Parser)
                               -> Result<DeclaredValue<SpecifiedValue>, ()> {
            let value = try!(CSSColor::parse(input));
            let rgba = match value.parsed {
                Color::RGBA(rgba) => rgba,
                Color::CurrentColor => return Ok(DeclaredValue::Inherit)
            };
            Ok(DeclaredValue::Value(CSSRGBA {
                parsed: rgba,
                authored: value.authored,
            }))
        }
    </%self:raw_longhand>

    // CSS 2.1, Section 15 - Fonts

    ${new_style_struct("Font", is_inherited=True, gecko_name="nsStyleFont",
                       additional_methods=[Method("clone_font_size",
                                                  "longhands::font_size::computed_value::T"),
                                           Method("clone_font_weight",
                                                  "longhands::font_weight::computed_value::T"),
                                           Method("compute_font_hash", is_mut=True)])}

    <%self:longhand name="font-family">
        use self::computed_value::FontFamily;
        use values::computed::ComputedValueAsSpecified;
        pub use self::computed_value::T as SpecifiedValue;

        const SERIF: &'static str = "serif";
        const SANS_SERIF: &'static str = "sans-serif";
        const CURSIVE: &'static str = "cursive";
        const FANTASY: &'static str = "fantasy";
        const MONOSPACE: &'static str = "monospace";

        impl ComputedValueAsSpecified for SpecifiedValue {}
        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;
            use string_cache::Atom;

            #[derive(Debug, PartialEq, Eq, Clone, Hash, HeapSizeOf, Deserialize, Serialize)]
            pub enum FontFamily {
                FamilyName(Atom),
                // Generic,
                Serif,
                SansSerif,
                Cursive,
                Fantasy,
                Monospace,
            }
            impl FontFamily {
                #[inline]
                pub fn name(&self) -> &str {
                    match *self {
                        FontFamily::FamilyName(ref name) => &*name,
                        FontFamily::Serif => super::SERIF,
                        FontFamily::SansSerif => super::SANS_SERIF,
                        FontFamily::Cursive => super::CURSIVE,
                        FontFamily::Fantasy => super::FANTASY,
                        FontFamily::Monospace => super::MONOSPACE
                    }
                }

                pub fn from_atom(input: Atom) -> FontFamily {
                    let option = match_ignore_ascii_case! { &input,
                        super::SERIF => Some(FontFamily::Serif),
                        super::SANS_SERIF => Some(FontFamily::SansSerif),
                        super::CURSIVE => Some(FontFamily::Cursive),
                        super::FANTASY => Some(FontFamily::Fantasy),
                        super::MONOSPACE => Some(FontFamily::Monospace),
                        _ => None
                    };

                    match option {
                        Some(family) => family,
                        None => FontFamily::FamilyName(input)
                    }
                }
            }
            impl ToCss for FontFamily {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    dest.write_str(self.name())
                }
            }
            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    let mut iter = self.0.iter();
                    try!(iter.next().unwrap().to_css(dest));
                    for family in iter {
                        try!(dest.write_str(", "));
                        try!(family.to_css(dest));
                    }
                    Ok(())
                }
            }
            #[derive(Debug, Clone, PartialEq, Eq, Hash, HeapSizeOf)]
            pub struct T(pub Vec<FontFamily>);
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(vec![FontFamily::Serif])
        }
        /// <family-name>#
        /// <family-name> = <string> | [ <ident>+ ]
        /// TODO: <generic-family>
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            input.parse_comma_separated(parse_one_family).map(SpecifiedValue)
        }
        pub fn parse_one_family(input: &mut Parser) -> Result<FontFamily, ()> {
            if let Ok(value) = input.try(|input| input.expect_string()) {
                return Ok(FontFamily::FamilyName(Atom::from(&*value)))
            }
            let first_ident = try!(input.expect_ident());

            match_ignore_ascii_case! { first_ident,
                SERIF => return Ok(FontFamily::Serif),
                SANS_SERIF => return Ok(FontFamily::SansSerif),
                CURSIVE => return Ok(FontFamily::Cursive),
                FANTASY => return Ok(FontFamily::Fantasy),
                MONOSPACE => return Ok(FontFamily::Monospace),
                _ => {}
            }
            let mut value = first_ident.into_owned();
            while let Ok(ident) = input.try(|input| input.expect_ident()) {
                value.push_str(" ");
                value.push_str(&ident);
            }
            Ok(FontFamily::FamilyName(Atom::from(value)))
        }
    </%self:longhand>


    ${single_keyword("font-style", "normal italic oblique")}
    ${single_keyword("font-variant", "normal small-caps")}

    <%self:longhand name="font-weight">
        use cssparser::ToCss;
        use std::fmt;

        #[derive(Debug, Clone, PartialEq, Eq, Copy, HeapSizeOf)]
        pub enum SpecifiedValue {
            Bolder,
            Lighter,
            % for weight in range(100, 901, 100):
                Weight${weight},
            % endfor
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Bolder => dest.write_str("bolder"),
                    SpecifiedValue::Lighter => dest.write_str("lighter"),
                    % for weight in range(100, 901, 100):
                        SpecifiedValue::Weight${weight} => dest.write_str("${weight}"),
                    % endfor
                }
            }
        }
        /// normal | bold | bolder | lighter | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            input.try(|input| {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    "bold" => Ok(SpecifiedValue::Weight700),
                    "normal" => Ok(SpecifiedValue::Weight400),
                    "bolder" => Ok(SpecifiedValue::Bolder),
                    "lighter" => Ok(SpecifiedValue::Lighter),
                    _ => Err(())
                }
            }).or_else(|()| {
                match try!(input.expect_integer()) {
                    100 => Ok(SpecifiedValue::Weight100),
                    200 => Ok(SpecifiedValue::Weight200),
                    300 => Ok(SpecifiedValue::Weight300),
                    400 => Ok(SpecifiedValue::Weight400),
                    500 => Ok(SpecifiedValue::Weight500),
                    600 => Ok(SpecifiedValue::Weight600),
                    700 => Ok(SpecifiedValue::Weight700),
                    800 => Ok(SpecifiedValue::Weight800),
                    900 => Ok(SpecifiedValue::Weight900),
                    _ => Err(())
                }
            })
        }
        pub mod computed_value {
            use std::fmt;
            #[derive(PartialEq, Eq, Copy, Clone, Hash, Deserialize, Serialize, HeapSizeOf, Debug)]
            pub enum T {
                % for weight in range(100, 901, 100):
                    Weight${weight} = ${weight},
                % endfor
            }
            impl T {
                #[inline]
                pub fn is_bold(self) -> bool {
                    match self {
                        T::Weight900 | T::Weight800 |
                        T::Weight700 | T::Weight600 => true,
                        _ => false
                    }
                }
            }
        }
        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    % for weight in range(100, 901, 100):
                        computed_value::T::Weight${weight} => dest.write_str("${weight}"),
                    % endfor
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::Weight400  // normal
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                match *self {
                    % for weight in range(100, 901, 100):
                        SpecifiedValue::Weight${weight} => computed_value::T::Weight${weight},
                    % endfor
                    SpecifiedValue::Bolder => match context.inherited_style().get_font().clone_font_weight() {
                        computed_value::T::Weight100 => computed_value::T::Weight400,
                        computed_value::T::Weight200 => computed_value::T::Weight400,
                        computed_value::T::Weight300 => computed_value::T::Weight400,
                        computed_value::T::Weight400 => computed_value::T::Weight700,
                        computed_value::T::Weight500 => computed_value::T::Weight700,
                        computed_value::T::Weight600 => computed_value::T::Weight900,
                        computed_value::T::Weight700 => computed_value::T::Weight900,
                        computed_value::T::Weight800 => computed_value::T::Weight900,
                        computed_value::T::Weight900 => computed_value::T::Weight900,
                    },
                    SpecifiedValue::Lighter => match context.inherited_style().get_font().clone_font_weight() {
                        computed_value::T::Weight100 => computed_value::T::Weight100,
                        computed_value::T::Weight200 => computed_value::T::Weight100,
                        computed_value::T::Weight300 => computed_value::T::Weight100,
                        computed_value::T::Weight400 => computed_value::T::Weight100,
                        computed_value::T::Weight500 => computed_value::T::Weight100,
                        computed_value::T::Weight600 => computed_value::T::Weight400,
                        computed_value::T::Weight700 => computed_value::T::Weight400,
                        computed_value::T::Weight800 => computed_value::T::Weight700,
                        computed_value::T::Weight900 => computed_value::T::Weight700,
                    },
                }
            }
        }
    </%self:longhand>

    <%self:longhand name="font-size">
        use app_units::Au;
        use cssparser::ToCss;
        use std::fmt;
        use values::FONT_MEDIUM_PX;
        use values::specified::{LengthOrPercentage, Length, Percentage};

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.0.to_css(dest)
            }
        }

        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct SpecifiedValue(pub specified::LengthOrPercentage);
        pub mod computed_value {
            use app_units::Au;
            pub type T = Au;
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            Au::from_px(FONT_MEDIUM_PX)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                match self.0 {
                    LengthOrPercentage::Length(Length::FontRelative(value)) => {
                        value.to_computed_value(context.inherited_style().get_font().clone_font_size(),
                                                context.style().root_font_size())
                    }
                    LengthOrPercentage::Length(Length::ServoCharacterWidth(value)) => {
                        value.to_computed_value(context.inherited_style().get_font().clone_font_size())
                    }
                    LengthOrPercentage::Length(l) => {
                        l.to_computed_value(context)
                    }
                    LengthOrPercentage::Percentage(Percentage(value)) => {
                        context.inherited_style().get_font().clone_font_size().scale_by(value)
                    }
                    LengthOrPercentage::Calc(calc) => {
                        let calc = calc.to_computed_value(context);
                        calc.length() + context.inherited_style().get_font().clone_font_size()
                                               .scale_by(calc.percentage())
                    }
                }
            }
        }
        /// <length> | <percentage> | <absolute-size> | <relative-size>
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            use values::specified::{Length, LengthOrPercentage};

            input.try(specified::LengthOrPercentage::parse_non_negative)
            .or_else(|()| {
                let ident = try!(input.expect_ident());
                specified::Length::from_str(&ident as &str)
                    .ok_or(())
                    .map(specified::LengthOrPercentage::Length)
            })
            .map(SpecifiedValue)
        }
    </%self:longhand>

    ${single_keyword("font-stretch",
                     "normal ultra-condensed extra-condensed condensed semi-condensed semi-expanded \
                     expanded extra-expanded ultra-expanded")}

    // CSS 2.1, Section 16 - Text

    ${switch_to_style_struct("InheritedText")}

    <%self:longhand name="text-align">
        pub use self::computed_value::T as SpecifiedValue;
        use values::computed::ComputedValueAsSpecified;
        impl ComputedValueAsSpecified for SpecifiedValue {}
        pub mod computed_value {
            macro_rules! define_text_align {
                ( $( $name: ident ( $string: expr ) => $discriminant: expr, )+ ) => {
                    define_css_keyword_enum! { T:
                        $(
                            $string => $name,
                        )+
                    }
                    impl T {
                        pub fn to_u32(self) -> u32 {
                            match self {
                                $(
                                    T::$name => $discriminant,
                                )+
                            }
                        }
                        pub fn from_u32(discriminant: u32) -> Option<T> {
                            match discriminant {
                                $(
                                    $discriminant => Some(T::$name),
                                )+
                                _ => None
                            }
                        }
                    }
                }
            }
            define_text_align! {
                start("start") => 0,
                end("end") => 1,
                left("left") => 2,
                right("right") => 3,
                center("center") => 4,
                justify("justify") => 5,
                servo_center("-servo-center") => 6,
                servo_left("-servo-left") => 7,
                servo_right("-servo-right") => 8,
            }
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            computed_value::T::start
        }
        pub fn parse(_context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            computed_value::T::parse(input)
        }
    </%self:longhand>

    <%self:longhand name="letter-spacing">
        use cssparser::ToCss;
        use std::fmt;
        use values::AuExtensionMethods;

        #[derive(Debug, Clone, Copy, PartialEq, HeapSizeOf)]
        pub enum SpecifiedValue {
            Normal,
            Specified(specified::Length),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Normal => dest.write_str("normal"),
                    SpecifiedValue::Specified(l) => l.to_css(dest),
                }
            }
        }

        pub mod computed_value {
            use app_units::Au;
            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Au>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("normal"),
                    Some(l) => l.to_css(dest),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                match *self {
                    SpecifiedValue::Normal => computed_value::T(None),
                    SpecifiedValue::Specified(l) =>
                        computed_value::T(Some(l.to_computed_value(context)))
                }
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                Ok(SpecifiedValue::Normal)
            } else {
                specified::Length::parse_non_negative(input).map(SpecifiedValue::Specified)
            }
        }
    </%self:longhand>

    <%self:longhand name="word-spacing">
        use cssparser::ToCss;
        use std::fmt;
        use values::AuExtensionMethods;

        #[derive(Debug, Clone, Copy, PartialEq, HeapSizeOf)]
        pub enum SpecifiedValue {
            Normal,
            Specified(specified::Length),  // FIXME(SimonSapin) support percentages
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Normal => dest.write_str("normal"),
                    SpecifiedValue::Specified(l) => l.to_css(dest),
                }
            }
        }

        pub mod computed_value {
            use app_units::Au;
            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Au>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("normal"),
                    Some(l) => l.to_css(dest),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                match *self {
                    SpecifiedValue::Normal => computed_value::T(None),
                    SpecifiedValue::Specified(l) =>
                        computed_value::T(Some(l.to_computed_value(context)))
                }
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                Ok(SpecifiedValue::Normal)
            } else {
                specified::Length::parse_non_negative(input).map(SpecifiedValue::Specified)
            }
        }
    </%self:longhand>

    ${predefined_type("text-indent", "LengthOrPercentage", "computed::LengthOrPercentage::Length(Au(0))")}

    // Also known as "word-wrap" (which is more popular because of IE), but this is the preferred
    // name per CSS-TEXT 6.2.
    ${single_keyword("overflow-wrap", "normal break-word")}

    // TODO(pcwalton): Support `word-break: keep-all` once we have better CJK support.
    ${single_keyword("word-break", "normal break-all")}

    // TODO(pcwalton): Support `text-justify: distribute`.
    ${single_keyword("text-justify", "auto none inter-word")}

    ${new_style_struct("Text", is_inherited=False, gecko_name="nsStyleTextReset",
                       additional_methods=[Method("has_underline", "bool"),
                                           Method("has_overline", "bool"),
                                           Method("has_line_through", "bool")])}

    ${single_keyword("text-overflow", "clip ellipsis")}

    ${single_keyword("unicode-bidi", "normal embed isolate bidi-override isolate-override plaintext")}

    <%self:longhand name="text-decoration" custom_cascade="True">
        use cssparser::ToCss;
        use std::fmt;
        use values::computed::ComputedValueAsSpecified;
        use properties::style_struct_traits::TInheritedText;

        impl ComputedValueAsSpecified for SpecifiedValue {}

        #[derive(PartialEq, Eq, Copy, Clone, Debug, HeapSizeOf)]
        pub struct SpecifiedValue {
            pub underline: bool,
            pub overline: bool,
            pub line_through: bool,
            // 'blink' is accepted in the parser but ignored.
            // Just not blinking the text is a conforming implementation per CSS 2.1.
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut space = false;
                if self.underline {
                    try!(dest.write_str("underline"));
                    space = true;
                }
                if self.overline {
                    if space {
                        try!(dest.write_str(" "));
                    }
                    try!(dest.write_str("overline"));
                    space = true;
                }
                if self.line_through {
                    if space {
                        try!(dest.write_str(" "));
                    }
                    try!(dest.write_str("line-through"));
                }
                Ok(())
            }
        }
        pub mod computed_value {
            pub type T = super::SpecifiedValue;
            #[allow(non_upper_case_globals)]
            pub const none: T = super::SpecifiedValue {
                underline: false, overline: false, line_through: false
            };
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            computed_value::none
        }
        /// none | [ underline || overline || line-through || blink ]
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            let mut result = SpecifiedValue {
                underline: false, overline: false, line_through: false,
            };
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(result)
            }
            let mut blink = false;
            let mut empty = true;
            while let Ok(ident) = input.expect_ident() {
                match_ignore_ascii_case! { ident,
                    "underline" => if result.underline { return Err(()) }
                                  else { empty = false; result.underline = true },
                    "overline" => if result.overline { return Err(()) }
                                  else { empty = false; result.overline = true },
                    "line-through" => if result.line_through { return Err(()) }
                                      else { empty = false; result.line_through = true },
                    "blink" => if blink { return Err(()) }
                               else { empty = false; blink = true },
                    _ => break
                }
            }
            if !empty { Ok(result) } else { Err(()) }
        }

        fn cascade_property_custom<C: ComputedValues>(
                                   _declaration: &PropertyDeclaration,
                                   _inherited_style: &C,
                                   context: &mut computed::Context<C>,
                                   _seen: &mut PropertyBitField,
                                   _cacheable: &mut bool,
                                   _error_reporter: &mut Box<ParseErrorReporter + Send>) {
            longhands::_servo_text_decorations_in_effect::derive_from_text_decoration(context);
        }
    </%self:longhand>

    ${switch_to_style_struct("InheritedText")}

    <%self:longhand name="-servo-text-decorations-in-effect"
                    derived_from="display text-decoration">
        use cssparser::{RGBA, ToCss};
        use std::fmt;

        use values::computed::ComputedValueAsSpecified;
        use properties::style_struct_traits::{TBox, TColor, TText};

        impl ComputedValueAsSpecified for SpecifiedValue {}

        #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
        pub struct SpecifiedValue {
            pub underline: Option<RGBA>,
            pub overline: Option<RGBA>,
            pub line_through: Option<RGBA>,
        }

        pub mod computed_value {
            pub type T = super::SpecifiedValue;
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
                // Web compat doesn't matter here.
                Ok(())
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            SpecifiedValue {
                underline: None,
                overline: None,
                line_through: None,
            }
        }

        fn maybe<Cx: TContext>(flag: bool, context: &Cx) -> Option<RGBA> {
            if flag {
                Some(context.style().get_color().clone_color())
            } else {
                None
            }
        }

        fn derive<Cx: TContext>(context: &Cx) -> computed_value::T {
            // Start with no declarations if this is an atomic inline-level box; otherwise, start with the
            // declarations in effect and add in the text decorations that this block specifies.
            let mut result = match context.style().get_box().clone_display() {
                super::display::computed_value::T::inline_block |
                super::display::computed_value::T::inline_table => SpecifiedValue {
                    underline: None,
                    overline: None,
                    line_through: None,
                },
                _ => context.inherited_style().get_inheritedtext().clone__servo_text_decorations_in_effect()
            };

            result.underline = maybe(context.style().get_text().has_underline()
                                     || result.underline.is_some(), context);
            result.overline = maybe(context.style().get_text().has_overline()
                                    || result.overline.is_some(), context);
            result.line_through = maybe(context.style().get_text().has_line_through()
                                        || result.line_through.is_some(), context);

            result
        }

        #[inline]
        pub fn derive_from_text_decoration<Cx: TContext>(context: &mut Cx) {
            let derived = derive(context);
            context.mutate_style().mutate_inheritedtext().set__servo_text_decorations_in_effect(derived);
        }

        #[inline]
        pub fn derive_from_display<Cx: TContext>(context: &mut Cx) {
            let derived = derive(context);
            context.mutate_style().mutate_inheritedtext().set__servo_text_decorations_in_effect(derived);
        }
    </%self:longhand>

    <%self:single_keyword_computed name="white-space" values="normal pre nowrap pre-wrap pre-line">
        use values::computed::ComputedValueAsSpecified;
        impl ComputedValueAsSpecified for SpecifiedValue {}

        impl SpecifiedValue {
            pub fn allow_wrap(&self) -> bool {
                match *self {
                    SpecifiedValue::nowrap |
                    SpecifiedValue::pre => false,
                    SpecifiedValue::normal |
                    SpecifiedValue::pre_wrap |
                    SpecifiedValue::pre_line => true,
                }
            }

            pub fn preserve_newlines(&self) -> bool {
                match *self {
                    SpecifiedValue::normal |
                    SpecifiedValue::nowrap => false,
                    SpecifiedValue::pre |
                    SpecifiedValue::pre_wrap |
                    SpecifiedValue::pre_line => true,
                }
            }

            pub fn preserve_spaces(&self) -> bool {
                match *self {
                    SpecifiedValue::normal |
                    SpecifiedValue::nowrap |
                    SpecifiedValue::pre_line => false,
                    SpecifiedValue::pre |
                    SpecifiedValue::pre_wrap => true,
                }
            }
        }
    </%self:single_keyword_computed>

    // TODO(pcwalton): `full-width`
    ${single_keyword("text-transform", "none capitalize uppercase lowercase")}

    ${single_keyword("text-rendering", "auto optimizespeed optimizelegibility geometricprecision")}

    // CSS 2.1, Section 17 - Tables
    ${new_style_struct("Table", is_inherited=False, gecko_name="nsStyleTable")}

    ${single_keyword("table-layout", "auto fixed")}

    ${new_style_struct("InheritedTable", is_inherited=True)}

    ${single_keyword("border-collapse", "separate collapse")}

    ${single_keyword("empty-cells", "show hide")}

    ${single_keyword("caption-side", "top bottom")}

    <%self:longhand name="border-spacing">
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
    </%self:longhand>

    // CSS 2.1, Section 18 - User interface


    // CSS Writing Modes Level 3
    // http://dev.w3.org/csswg/css-writing-modes/
    ${switch_to_style_struct("InheritedBox")}

    ${single_keyword("writing-mode", "horizontal-tb vertical-rl vertical-lr", experimental=True)}

    // FIXME(SimonSapin): Add 'mixed' and 'upright' (needs vertical text support)
    // FIXME(SimonSapin): initial (first) value should be 'mixed', when that's implemented
    ${single_keyword("text-orientation", "sideways sideways-left sideways-right", experimental=True)}

    // CSS Basic User Interface Module Level 3
    // http://dev.w3.org/csswg/css-ui/
    ${switch_to_style_struct("Position")}

    ${single_keyword("box-sizing", "content-box border-box")}

    ${new_style_struct("Pointing", is_inherited=True)}

    <%self:longhand name="cursor">
        pub use self::computed_value::T as SpecifiedValue;
        use values::computed::ComputedValueAsSpecified;

        impl ComputedValueAsSpecified for SpecifiedValue {}

        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;
            use style_traits::cursor::Cursor;

            #[derive(Clone, PartialEq, Eq, Copy, Debug, HeapSizeOf)]
            pub enum T {
                AutoCursor,
                SpecifiedCursor(Cursor),
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        T::AutoCursor => dest.write_str("auto"),
                        T::SpecifiedCursor(c) => c.to_css(dest),
                    }
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::AutoCursor
        }
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            use std::ascii::AsciiExt;
            use style_traits::cursor::Cursor;
            let ident = try!(input.expect_ident());
            if ident.eq_ignore_ascii_case("auto") {
                Ok(SpecifiedValue::AutoCursor)
            } else {
                Cursor::from_css_keyword(&ident)
                .map(SpecifiedValue::SpecifiedCursor)
            }
        }
    </%self:longhand>

    // NB: `pointer-events: auto` (and use of `pointer-events` in anything that isn't SVG, in fact)
    // is nonstandard, slated for CSS4-UI.
    // TODO(pcwalton): SVG-only values.
    ${single_keyword("pointer-events", "auto none")}


    ${new_style_struct("Column", is_inherited=False, gecko_name="nsStyleColumn")}

    <%self:longhand name="column-width" experimental="True">
        use cssparser::ToCss;
        use std::fmt;
        use values::AuExtensionMethods;

        #[derive(Debug, Clone, Copy, PartialEq, HeapSizeOf)]
        pub enum SpecifiedValue {
            Auto,
            Specified(specified::Length),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Auto => dest.write_str("auto"),
                    SpecifiedValue::Specified(l) => l.to_css(dest),
                }
            }
        }

        pub mod computed_value {
            use app_units::Au;
            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Au>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("auto"),
                    Some(l) => l.to_css(dest),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                match *self {
                    SpecifiedValue::Auto => computed_value::T(None),
                    SpecifiedValue::Specified(l) =>
                        computed_value::T(Some(l.to_computed_value(context)))
                }
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                Ok(SpecifiedValue::Auto)
            } else {
                specified::Length::parse_non_negative(input).map(SpecifiedValue::Specified)
            }
        }
    </%self:longhand>

    <%self:longhand name="column-count" experimental="True">
        use cssparser::ToCss;
        use std::fmt;

        #[derive(Debug, Clone, Copy, PartialEq, HeapSizeOf)]
        pub enum SpecifiedValue {
            Auto,
            Specified(u32),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Auto => dest.write_str("auto"),
                    SpecifiedValue::Specified(count) => write!(dest, "{}", count),
                }
            }
        }

        pub mod computed_value {
            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<u32>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("auto"),
                    Some(count) => write!(dest, "{}", count),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, _context: &Cx) -> computed_value::T {
                match *self {
                    SpecifiedValue::Auto => computed_value::T(None),
                    SpecifiedValue::Specified(count) =>
                        computed_value::T(Some(count))
                }
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                Ok(SpecifiedValue::Auto)
            } else {
                let count = try!(specified::parse_integer(input));
                // Zero is invalid
                if count <= 0 {
                    return Err(())
                }
                Ok(SpecifiedValue::Specified(count as u32))
            }
        }
    </%self:longhand>

    <%self:longhand name="column-gap" experimental="True">
        use cssparser::ToCss;
        use std::fmt;
        use values::AuExtensionMethods;

        #[derive(Debug, Clone, Copy, PartialEq, HeapSizeOf)]
        pub enum SpecifiedValue {
            Normal,
            Specified(specified::Length),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Normal => dest.write_str("normal"),
                    SpecifiedValue::Specified(l) => l.to_css(dest),
                }
            }
        }

        pub mod computed_value {
            use app_units::Au;
            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Au>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("normal"),
                    Some(l) => l.to_css(dest),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                match *self {
                    SpecifiedValue::Normal => computed_value::T(None),
                    SpecifiedValue::Specified(l) =>
                        computed_value::T(Some(l.to_computed_value(context)))
                }
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                Ok(SpecifiedValue::Normal)
            } else {
                specified::Length::parse_non_negative(input).map(SpecifiedValue::Specified)
            }
        }
    </%self:longhand>

    // Box-shadow, etc.
    ${new_style_struct("Effects", is_inherited=False)}

    <%self:longhand name="opacity">
        use cssparser::ToCss;
        use std::fmt;
        use values::CSSFloat;

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.0.to_css(dest)
            }
        }

        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct SpecifiedValue(pub CSSFloat);
        pub mod computed_value {
            use values::CSSFloat;
            pub type T = CSSFloat;
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            1.0
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, _context: &Cx) -> computed_value::T {
                if self.0 < 0.0 {
                    0.0
                } else if self.0 > 1.0 {
                    1.0
                } else {
                    self.0
                }
            }
        }
        fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            specified::parse_number(input).map(SpecifiedValue)
        }
    </%self:longhand>

    <%self:longhand name="box-shadow">
        use cssparser::{self, ToCss};
        use std::fmt;
        use values::AuExtensionMethods;

        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct SpecifiedValue(Vec<SpecifiedBoxShadow>);

        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct SpecifiedBoxShadow {
            pub offset_x: specified::Length,
            pub offset_y: specified::Length,
            pub blur_radius: specified::Length,
            pub spread_radius: specified::Length,
            pub color: Option<specified::CSSColor>,
            pub inset: bool,
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
                if let Some(shadow) = iter.next() {
                    try!(shadow.to_css(dest));
                } else {
                    try!(dest.write_str("none"));
                    return Ok(())
                }
                for shadow in iter {
                    try!(dest.write_str(", "));
                    try!(shadow.to_css(dest));
                }
                Ok(())
            }
        }

        impl ToCss for SpecifiedBoxShadow {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                if self.inset {
                    try!(dest.write_str("inset "));
                }
                try!(self.blur_radius.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.spread_radius.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.offset_x.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.offset_y.to_css(dest));

                if let Some(ref color) = self.color {
                    try!(dest.write_str(" "));
                    try!(color.to_css(dest));
                }
                Ok(())
            }
        }

        pub mod computed_value {
            use app_units::Au;
            use std::fmt;
            use values::computed;

            #[derive(Clone, PartialEq, HeapSizeOf, Debug)]
            pub struct T(pub Vec<BoxShadow>);

            #[derive(Clone, PartialEq, Copy, HeapSizeOf, Debug)]
            pub struct BoxShadow {
                pub offset_x: Au,
                pub offset_y: Au,
                pub blur_radius: Au,
                pub spread_radius: Au,
                pub color: computed::CSSColor,
                pub inset: bool,
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
                if let Some(shadow) = iter.next() {
                    try!(shadow.to_css(dest));
                } else {
                    try!(dest.write_str("none"));
                    return Ok(())
                }
                for shadow in iter {
                    try!(dest.write_str(", "));
                    try!(shadow.to_css(dest));
                }
                Ok(())
            }
        }

        impl ToCss for computed_value::BoxShadow {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                if self.inset {
                    try!(dest.write_str("inset "));
                }
                try!(self.blur_radius.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.spread_radius.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.offset_x.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.offset_y.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.color.to_css(dest));
                Ok(())
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(Vec::new())
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                Ok(SpecifiedValue(Vec::new()))
            } else {
                input.parse_comma_separated(parse_one_box_shadow).map(SpecifiedValue)
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                computed_value::T(self.0.iter().map(|value| compute_one_box_shadow(value, context)).collect())
            }
        }

        pub fn compute_one_box_shadow<Cx: TContext>(value: &SpecifiedBoxShadow, context: &Cx)
                                      -> computed_value::BoxShadow {
            computed_value::BoxShadow {
                offset_x: value.offset_x.to_computed_value(context),
                offset_y: value.offset_y.to_computed_value(context),
                blur_radius: value.blur_radius.to_computed_value(context),
                spread_radius: value.spread_radius.to_computed_value(context),
                color: value.color
                            .as_ref()
                            .map(|color| color.parsed)
                            .unwrap_or(cssparser::Color::CurrentColor),
                inset: value.inset,
            }
        }

        pub fn parse_one_box_shadow(input: &mut Parser) -> Result<SpecifiedBoxShadow, ()> {
            use app_units::Au;
            let mut lengths = [specified::Length::Absolute(Au(0)); 4];
            let mut lengths_parsed = false;
            let mut color = None;
            let mut inset = false;

            loop {
                if !inset {
                    if input.try(|input| input.expect_ident_matching("inset")).is_ok() {
                        inset = true;
                        continue
                    }
                }
                if !lengths_parsed {
                    if let Ok(value) = input.try(specified::Length::parse) {
                        lengths[0] = value;
                        let mut length_parsed_count = 1;
                        while length_parsed_count < 4 {
                            if let Ok(value) = input.try(specified::Length::parse) {
                                lengths[length_parsed_count] = value
                            } else {
                                break
                            }
                            length_parsed_count += 1;
                        }

                        // The first two lengths must be specified.
                        if length_parsed_count < 2 {
                            return Err(())
                        }

                        lengths_parsed = true;
                        continue
                    }
                }
                if color.is_none() {
                    if let Ok(value) = input.try(specified::CSSColor::parse) {
                        color = Some(value);
                        continue
                    }
                }
                break
            }

            // Lengths must be specified.
            if !lengths_parsed {
                return Err(())
            }

            Ok(SpecifiedBoxShadow {
                offset_x: lengths[0],
                offset_y: lengths[1],
                blur_radius: lengths[2],
                spread_radius: lengths[3],
                color: color,
                inset: inset,
            })
        }
    </%self:longhand>

    <%self:longhand name="clip">
        use cssparser::ToCss;
        use std::fmt;
        use values::AuExtensionMethods;

        // NB: `top` and `left` are 0 if `auto` per CSS 2.1 11.1.2.

        pub mod computed_value {
            use app_units::Au;

            #[derive(Clone, PartialEq, Eq, Copy, Debug, HeapSizeOf)]
            pub struct ClipRect {
                pub top: Au,
                pub right: Option<Au>,
                pub bottom: Option<Au>,
                pub left: Au,
            }

            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<ClipRect>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("auto"),
                    Some(rect) => {
                        try!(dest.write_str("rect("));
                        try!(rect.top.to_css(dest));
                        try!(dest.write_str(", "));
                        if let Some(right) = rect.right {
                            try!(right.to_css(dest));
                            try!(dest.write_str(", "));
                        } else {
                            try!(dest.write_str("auto, "));
                        }

                        if let Some(bottom) = rect.bottom {
                            try!(bottom.to_css(dest));
                            try!(dest.write_str(", "));
                        } else {
                            try!(dest.write_str("auto, "));
                        }

                        try!(rect.left.to_css(dest));
                        try!(dest.write_str(")"));
                        Ok(())
                    }
                }
            }
        }

        #[derive(Clone, Debug, PartialEq, Copy, HeapSizeOf)]
        pub struct SpecifiedClipRect {
            pub top: specified::Length,
            pub right: Option<specified::Length>,
            pub bottom: Option<specified::Length>,
            pub left: specified::Length,
        }

        #[derive(Clone, Debug, PartialEq, Copy, HeapSizeOf)]
        pub struct SpecifiedValue(Option<SpecifiedClipRect>);

        impl ToCss for SpecifiedClipRect {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(dest.write_str("rect("));

                try!(self.top.to_css(dest));
                try!(dest.write_str(", "));

                if let Some(right) = self.right {
                    try!(right.to_css(dest));
                    try!(dest.write_str(", "));
                } else {
                    try!(dest.write_str("auto, "));
                }

                if let Some(bottom) = self.bottom {
                    try!(bottom.to_css(dest));
                    try!(dest.write_str(", "));
                } else {
                    try!(dest.write_str("auto, "));
                }

                try!(self.left.to_css(dest));

                try!(dest.write_str(")"));
                Ok(())
            }
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                if let Some(ref rect) = self.0 {
                    rect.to_css(dest)
                } else {
                    dest.write_str("auto")
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                computed_value::T(self.0.map(|value| computed_value::ClipRect {
                    top: value.top.to_computed_value(context),
                    right: value.right.map(|right| right.to_computed_value(context)),
                    bottom: value.bottom.map(|bottom| bottom.to_computed_value(context)),
                    left: value.left.to_computed_value(context),
                }))
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            use app_units::Au;
            use std::ascii::AsciiExt;
            use values::specified::Length;

            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                return Ok(SpecifiedValue(None))
            }
            if !try!(input.expect_function()).eq_ignore_ascii_case("rect") {
                return Err(())
            }
            let sides = try!(input.parse_nested_block(|input| {
                input.parse_comma_separated(|input| {
                    if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                        Ok(None)
                    } else {
                        Length::parse(input).map(Some)
                    }
                })
            }));
            if sides.len() == 4 {
                Ok(SpecifiedValue(Some(SpecifiedClipRect {
                    top: sides[0].unwrap_or(Length::Absolute(Au(0))),
                    right: sides[1],
                    bottom: sides[2],
                    left: sides[3].unwrap_or(Length::Absolute(Au(0))),
                })))
            } else {
                Err(())
            }
        }
    </%self:longhand>

    ${switch_to_style_struct("InheritedText")}

    <%self:longhand name="text-shadow">
        use cssparser::{self, ToCss};
        use std::fmt;
        use values::AuExtensionMethods;

        #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
        pub struct SpecifiedValue(Vec<SpecifiedTextShadow>);

        #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
        pub struct SpecifiedTextShadow {
            pub offset_x: specified::Length,
            pub offset_y: specified::Length,
            pub blur_radius: specified::Length,
            pub color: Option<specified::CSSColor>,
        }

        pub mod computed_value {
            use app_units::Au;
            use cssparser::Color;

            #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
            pub struct T(pub Vec<TextShadow>);

            #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
            pub struct TextShadow {
                pub offset_x: Au,
                pub offset_y: Au,
                pub blur_radius: Au,
                pub color: Color,
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
                if let Some(shadow) = iter.next() {
                    try!(shadow.to_css(dest));
                } else {
                    try!(dest.write_str("none"));
                    return Ok(())
                }
                for shadow in iter {
                    try!(dest.write_str(", "));
                    try!(shadow.to_css(dest));
                }
                Ok(())
            }
        }

        impl ToCss for computed_value::TextShadow {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.offset_x.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.offset_y.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.blur_radius.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.color.to_css(dest));
                Ok(())
            }
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
                if let Some(shadow) = iter.next() {
                    try!(shadow.to_css(dest));
                } else {
                    try!(dest.write_str("none"));
                    return Ok(())
                }
                for shadow in iter {
                    try!(dest.write_str(", "));
                    try!(shadow.to_css(dest));
                }
                Ok(())
            }
        }

        impl ToCss for SpecifiedTextShadow {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.offset_x.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.offset_y.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.blur_radius.to_css(dest));

                if let Some(ref color) = self.color {
                    try!(dest.write_str(" "));
                    try!(color.to_css(dest));
                }
                Ok(())
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(Vec::new())
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                Ok(SpecifiedValue(Vec::new()))
            } else {
                input.parse_comma_separated(parse_one_text_shadow).map(SpecifiedValue)
            }
        }

        fn parse_one_text_shadow(input: &mut Parser) -> Result<SpecifiedTextShadow,()> {
            use app_units::Au;
            let mut lengths = [specified::Length::Absolute(Au(0)); 3];
            let mut lengths_parsed = false;
            let mut color = None;

            loop {
                if !lengths_parsed {
                    if let Ok(value) = input.try(specified::Length::parse) {
                        lengths[0] = value;
                        let mut length_parsed_count = 1;
                        while length_parsed_count < 3 {
                            if let Ok(value) = input.try(specified::Length::parse) {
                                lengths[length_parsed_count] = value
                            } else {
                                break
                            }
                            length_parsed_count += 1;
                        }

                        // The first two lengths must be specified.
                        if length_parsed_count < 2 {
                            return Err(())
                        }

                        lengths_parsed = true;
                        continue
                    }
                }
                if color.is_none() {
                    if let Ok(value) = input.try(specified::CSSColor::parse) {
                        color = Some(value);
                        continue
                    }
                }
                break
            }

            // Lengths must be specified.
            if !lengths_parsed {
                return Err(())
            }

            Ok(SpecifiedTextShadow {
                offset_x: lengths[0],
                offset_y: lengths[1],
                blur_radius: lengths[2],
                color: color,
            })
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                computed_value::T(self.0.iter().map(|value| {
                    computed_value::TextShadow {
                        offset_x: value.offset_x.to_computed_value(context),
                        offset_y: value.offset_y.to_computed_value(context),
                        blur_radius: value.blur_radius.to_computed_value(context),
                        color: value.color
                                    .as_ref()
                                    .map(|color| color.parsed)
                                    .unwrap_or(cssparser::Color::CurrentColor),
                    }
                }).collect())
            }
        }
    </%self:longhand>

    ${switch_to_style_struct("Effects")}

    <%self:longhand name="filter">
        //pub use self::computed_value::T as SpecifiedValue;
        use cssparser::ToCss;
        use std::fmt;
        use values::AuExtensionMethods;
        use values::CSSFloat;
        use values::specified::{Angle, Length};

        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct SpecifiedValue(Vec<SpecifiedFilter>);

        // TODO(pcwalton): `drop-shadow`
        #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
        pub enum SpecifiedFilter {
            Blur(Length),
            Brightness(CSSFloat),
            Contrast(CSSFloat),
            Grayscale(CSSFloat),
            HueRotate(Angle),
            Invert(CSSFloat),
            Opacity(CSSFloat),
            Saturate(CSSFloat),
            Sepia(CSSFloat),
        }

        pub mod computed_value {
            use app_units::Au;
            use values::CSSFloat;
            use values::specified::{Angle};

            #[derive(Clone, PartialEq, Debug, HeapSizeOf, Deserialize, Serialize)]
            pub enum Filter {
                Blur(Au),
                Brightness(CSSFloat),
                Contrast(CSSFloat),
                Grayscale(CSSFloat),
                HueRotate(Angle),
                Invert(CSSFloat),
                Opacity(CSSFloat),
                Saturate(CSSFloat),
                Sepia(CSSFloat),
            }

            #[derive(Clone, PartialEq, Debug, HeapSizeOf, Deserialize, Serialize)]
            pub struct T { pub filters: Vec<Filter> }

            impl T {
                /// Creates a new filter pipeline.
                #[inline]
                pub fn new(filters: Vec<Filter>) -> T {
                    T
                    {
                       filters: filters,
                    }
                }

                /// Adds a new filter to the filter pipeline.
                #[inline]
                pub fn push(&mut self, filter: Filter) {
                    self.filters.push(filter)
                }

                /// Returns true if this filter pipeline is empty and false otherwise.
                #[inline]
                pub fn is_empty(&self) -> bool {
                    self.filters.is_empty()
                }

                /// Returns the resulting opacity of this filter pipeline.
                #[inline]
                pub fn opacity(&self) -> CSSFloat {
                    let mut opacity = 1.0;

                    for filter in &self.filters {
                        if let Filter::Opacity(ref opacity_value) = *filter {
                            opacity *= *opacity_value
                        }
                    }
                    opacity
                }
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.filters.iter();
                if let Some(filter) = iter.next() {
                    try!(filter.to_css(dest));
                } else {
                    try!(dest.write_str("none"));
                    return Ok(())
                }
                for filter in iter {
                    try!(dest.write_str(" "));
                    try!(filter.to_css(dest));
                }
                Ok(())
            }
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
                if let Some(filter) = iter.next() {
                    try!(filter.to_css(dest));
                } else {
                    try!(dest.write_str("none"));
                    return Ok(())
                }
                for filter in iter {
                    try!(dest.write_str(" "));
                    try!(filter.to_css(dest));
                }
                Ok(())
            }
        }

        impl ToCss for computed_value::Filter {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    computed_value::Filter::Blur(value) => {
                        try!(dest.write_str("blur("));
                        try!(value.to_css(dest));
                        try!(dest.write_str(")"));
                    }
                    computed_value::Filter::Brightness(value) => try!(write!(dest, "brightness({})", value)),
                    computed_value::Filter::Contrast(value) => try!(write!(dest, "contrast({})", value)),
                    computed_value::Filter::Grayscale(value) => try!(write!(dest, "grayscale({})", value)),
                    computed_value::Filter::HueRotate(value) => {
                        try!(dest.write_str("hue-rotate("));
                        try!(value.to_css(dest));
                        try!(dest.write_str(")"));
                    }
                    computed_value::Filter::Invert(value) => try!(write!(dest, "invert({})", value)),
                    computed_value::Filter::Opacity(value) => try!(write!(dest, "opacity({})", value)),
                    computed_value::Filter::Saturate(value) => try!(write!(dest, "saturate({})", value)),
                    computed_value::Filter::Sepia(value) => try!(write!(dest, "sepia({})", value)),
                }
                Ok(())
            }
        }

        impl ToCss for SpecifiedFilter {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedFilter::Blur(value) => {
                        try!(dest.write_str("blur("));
                        try!(value.to_css(dest));
                        try!(dest.write_str(")"));
                    }
                    SpecifiedFilter::Brightness(value) => try!(write!(dest, "brightness({})", value)),
                    SpecifiedFilter::Contrast(value) => try!(write!(dest, "contrast({})", value)),
                    SpecifiedFilter::Grayscale(value) => try!(write!(dest, "grayscale({})", value)),
                    SpecifiedFilter::HueRotate(value) => {
                        try!(dest.write_str("hue-rotate("));
                        try!(value.to_css(dest));
                        try!(dest.write_str(")"));
                    }
                    SpecifiedFilter::Invert(value) => try!(write!(dest, "invert({})", value)),
                    SpecifiedFilter::Opacity(value) => try!(write!(dest, "opacity({})", value)),
                    SpecifiedFilter::Saturate(value) => try!(write!(dest, "saturate({})", value)),
                    SpecifiedFilter::Sepia(value) => try!(write!(dest, "sepia({})", value)),
                }
                Ok(())
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::new(Vec::new())
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            let mut filters = Vec::new();
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue(filters))
            }
            loop {
                if let Ok(function_name) = input.try(|input| input.expect_function()) {
                    filters.push(try!(input.parse_nested_block(|input| {
                        match_ignore_ascii_case! { function_name,
                            "blur" => specified::Length::parse_non_negative(input).map(SpecifiedFilter::Blur),
                            "brightness" => parse_factor(input).map(SpecifiedFilter::Brightness),
                            "contrast" => parse_factor(input).map(SpecifiedFilter::Contrast),
                            "grayscale" => parse_factor(input).map(SpecifiedFilter::Grayscale),
                            "hue-rotate" => Angle::parse(input).map(SpecifiedFilter::HueRotate),
                            "invert" => parse_factor(input).map(SpecifiedFilter::Invert),
                            "opacity" => parse_factor(input).map(SpecifiedFilter::Opacity),
                            "saturate" => parse_factor(input).map(SpecifiedFilter::Saturate),
                            "sepia" => parse_factor(input).map(SpecifiedFilter::Sepia),
                            _ => Err(())
                        }
                    })));
                } else if filters.is_empty() {
                    return Err(())
                } else {
                    return Ok(SpecifiedValue(filters))
                }
            }
        }

        fn parse_factor(input: &mut Parser) -> Result<::values::CSSFloat, ()> {
            use cssparser::Token;
            match input.next() {
                Ok(Token::Number(value)) => Ok(value.value),
                Ok(Token::Percentage(value)) => Ok(value.unit_value),
                _ => Err(())
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                computed_value::T{ filters: self.0.iter().map(|value| {
                    match *value {
                        SpecifiedFilter::Blur(factor) =>
                            computed_value::Filter::Blur(factor.to_computed_value(context)),
                        SpecifiedFilter::Brightness(factor) => computed_value::Filter::Brightness(factor),
                        SpecifiedFilter::Contrast(factor) => computed_value::Filter::Contrast(factor),
                        SpecifiedFilter::Grayscale(factor) => computed_value::Filter::Grayscale(factor),
                        SpecifiedFilter::HueRotate(factor) => computed_value::Filter::HueRotate(factor),
                        SpecifiedFilter::Invert(factor) => computed_value::Filter::Invert(factor),
                        SpecifiedFilter::Opacity(factor) => computed_value::Filter::Opacity(factor),
                        SpecifiedFilter::Saturate(factor) => computed_value::Filter::Saturate(factor),
                        SpecifiedFilter::Sepia(factor) => computed_value::Filter::Sepia(factor),
                    }
                }).collect() }
            }
        }
    </%self:longhand>

    <%self:longhand name="transform">
        use app_units::Au;
        use values::CSSFloat;

        use cssparser::ToCss;
        use std::fmt;

        pub mod computed_value {
            use values::CSSFloat;
            use values::computed;

            #[derive(Clone, Copy, Debug, PartialEq, HeapSizeOf)]
            pub struct ComputedMatrix {
                pub m11: CSSFloat, pub m12: CSSFloat, pub m13: CSSFloat, pub m14: CSSFloat,
                pub m21: CSSFloat, pub m22: CSSFloat, pub m23: CSSFloat, pub m24: CSSFloat,
                pub m31: CSSFloat, pub m32: CSSFloat, pub m33: CSSFloat, pub m34: CSSFloat,
                pub m41: CSSFloat, pub m42: CSSFloat, pub m43: CSSFloat, pub m44: CSSFloat,
            }

            impl ComputedMatrix {
                pub fn identity() -> ComputedMatrix {
                    ComputedMatrix {
                        m11: 1.0, m12: 0.0, m13: 0.0, m14: 0.0,
                        m21: 0.0, m22: 1.0, m23: 0.0, m24: 0.0,
                        m31: 0.0, m32: 0.0, m33: 1.0, m34: 0.0,
                        m41: 0.0, m42: 0.0, m43: 0.0, m44: 1.0
                    }
                }
            }

            #[derive(Clone, Debug, PartialEq, HeapSizeOf)]
            pub enum ComputedOperation {
                Matrix(ComputedMatrix),
                Skew(computed::Angle, computed::Angle),
                Translate(computed::LengthOrPercentage,
                          computed::LengthOrPercentage,
                          computed::Length),
                Scale(CSSFloat, CSSFloat, CSSFloat),
                Rotate(CSSFloat, CSSFloat, CSSFloat, computed::Angle),
                Perspective(computed::Length),
            }

            #[derive(Clone, Debug, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Vec<ComputedOperation>>);
        }

        pub use self::computed_value::ComputedMatrix as SpecifiedMatrix;

        fn parse_two_lengths_or_percentages(input: &mut Parser)
                                            -> Result<(specified::LengthOrPercentage,
                                                       specified::LengthOrPercentage),()> {
            let first = try!(specified::LengthOrPercentage::parse(input));
            let second = input.try(|input| {
                try!(input.expect_comma());
                specified::LengthOrPercentage::parse(input)
            }).unwrap_or(specified::LengthOrPercentage::zero());
            Ok((first, second))
        }

        fn parse_two_floats(input: &mut Parser) -> Result<(CSSFloat,CSSFloat),()> {
            let first = try!(specified::parse_number(input));
            let second = input.try(|input| {
                try!(input.expect_comma());
                specified::parse_number(input)
            }).unwrap_or(first);
            Ok((first, second))
        }

        fn parse_two_angles(input: &mut Parser) -> Result<(specified::Angle, specified::Angle),()> {
            let first = try!(specified::Angle::parse(input));
            let second = input.try(|input| {
                try!(input.expect_comma());
                specified::Angle::parse(input)
            }).unwrap_or(specified::Angle(0.0));
            Ok((first, second))
        }

        #[derive(Copy, Clone, Debug, PartialEq, HeapSizeOf)]
        enum TranslateKind {
            Translate,
            TranslateX,
            TranslateY,
            TranslateZ,
            Translate3D,
        }

        #[derive(Clone, Debug, PartialEq, HeapSizeOf)]
        enum SpecifiedOperation {
            Matrix(SpecifiedMatrix),
            Skew(specified::Angle, specified::Angle),
            Translate(TranslateKind,
                      specified::LengthOrPercentage,
                      specified::LengthOrPercentage,
                      specified::Length),
            Scale(CSSFloat, CSSFloat, CSSFloat),
            Rotate(CSSFloat, CSSFloat, CSSFloat, specified::Angle),
            Perspective(specified::Length),
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
                // TODO(pcwalton)
                Ok(())
            }
        }

        impl ToCss for SpecifiedOperation {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    // todo(gw): implement serialization for transform
                    // types other than translate.
                    SpecifiedOperation::Matrix(_m) => {
                        Ok(())
                    }
                    SpecifiedOperation::Skew(_sx, _sy) => {
                        Ok(())
                    }
                    SpecifiedOperation::Translate(kind, tx, ty, tz) => {
                        match kind {
                            TranslateKind::Translate => {
                                try!(dest.write_str("translate("));
                                try!(tx.to_css(dest));
                                try!(dest.write_str(", "));
                                try!(ty.to_css(dest));
                                dest.write_str(")")
                            }
                            TranslateKind::TranslateX => {
                                try!(dest.write_str("translateX("));
                                try!(tx.to_css(dest));
                                dest.write_str(")")
                            }
                            TranslateKind::TranslateY => {
                                try!(dest.write_str("translateY("));
                                try!(ty.to_css(dest));
                                dest.write_str(")")
                            }
                            TranslateKind::TranslateZ => {
                                try!(dest.write_str("translateZ("));
                                try!(tz.to_css(dest));
                                dest.write_str(")")
                            }
                            TranslateKind::Translate3D => {
                                try!(dest.write_str("translate3d("));
                                try!(tx.to_css(dest));
                                try!(dest.write_str(", "));
                                try!(ty.to_css(dest));
                                try!(dest.write_str(", "));
                                try!(tz.to_css(dest));
                                dest.write_str(")")
                            }
                        }
                    }
                    SpecifiedOperation::Scale(_sx, _sy, _sz) => {
                        Ok(())
                    }
                    SpecifiedOperation::Rotate(_ax, _ay, _az, _angle) => {
                        Ok(())
                    }
                    SpecifiedOperation::Perspective(_p) => {
                        Ok(())
                    }
                }
            }
        }

        #[derive(Clone, Debug, PartialEq, HeapSizeOf)]
        pub struct SpecifiedValue(Vec<SpecifiedOperation>);

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut first = true;
                for operation in &self.0 {
                    if !first {
                        try!(dest.write_str(" "));
                    }
                    first = false;
                    try!(operation.to_css(dest))
                }
                Ok(())
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue(Vec::new()))
            }

            let mut result = Vec::new();
            loop {
                let name = match input.expect_function() {
                    Ok(name) => name,
                    Err(_) => break,
                };
                match_ignore_ascii_case! {
                    name,
                    "matrix" => {
                        try!(input.parse_nested_block(|input| {
                            let values = try!(input.parse_comma_separated(|input| {
                                specified::parse_number(input)
                            }));
                            if values.len() != 6 {
                                return Err(())
                            }
                            result.push(SpecifiedOperation::Matrix(
                                    SpecifiedMatrix {
                                        m11: values[0], m12: values[1], m13: 0.0, m14: 0.0,
                                        m21: values[2], m22: values[3], m23: 0.0, m24: 0.0,
                                        m31:       0.0, m32:       0.0, m33: 1.0, m34: 0.0,
                                        m41: values[4], m42: values[5], m43: 0.0, m44: 1.0
                                    }));
                            Ok(())
                        }))
                    },
                    "matrix3d" => {
                        try!(input.parse_nested_block(|input| {
                            let values = try!(input.parse_comma_separated(|input| {
                                specified::parse_number(input)
                            }));
                            if values.len() != 16 {
                                return Err(())
                            }
                            result.push(SpecifiedOperation::Matrix(
                                    SpecifiedMatrix {
                                        m11: values[ 0], m12: values[ 1], m13: values[ 2], m14: values[ 3],
                                        m21: values[ 4], m22: values[ 5], m23: values[ 6], m24: values[ 7],
                                        m31: values[ 8], m32: values[ 9], m33: values[10], m34: values[11],
                                        m41: values[12], m42: values[13], m43: values[14], m44: values[15]
                                    }));
                            Ok(())
                        }))
                    },
                    "translate" => {
                        try!(input.parse_nested_block(|input| {
                            let (tx, ty) = try!(parse_two_lengths_or_percentages(input));
                            result.push(SpecifiedOperation::Translate(TranslateKind::Translate,
                                                                      tx,
                                                                      ty,
                                                                      specified::Length::Absolute(Au(0))));
                            Ok(())
                        }))
                    },
                    "translatex" => {
                        try!(input.parse_nested_block(|input| {
                            let tx = try!(specified::LengthOrPercentage::parse(input));
                            result.push(SpecifiedOperation::Translate(
                                TranslateKind::TranslateX,
                                tx,
                                specified::LengthOrPercentage::zero(),
                                specified::Length::Absolute(Au(0))));
                            Ok(())
                        }))
                    },
                    "translatey" => {
                        try!(input.parse_nested_block(|input| {
                            let ty = try!(specified::LengthOrPercentage::parse(input));
                            result.push(SpecifiedOperation::Translate(
                                TranslateKind::TranslateY,
                                specified::LengthOrPercentage::zero(),
                                ty,
                                specified::Length::Absolute(Au(0))));
                            Ok(())
                        }))
                    },
                    "translatez" => {
                        try!(input.parse_nested_block(|input| {
                            let tz = try!(specified::Length::parse(input));
                            result.push(SpecifiedOperation::Translate(
                                TranslateKind::TranslateZ,
                                specified::LengthOrPercentage::zero(),
                                specified::LengthOrPercentage::zero(),
                                tz));
                            Ok(())
                        }))
                    },
                    "translate3d" => {
                        try!(input.parse_nested_block(|input| {
                            let tx = try!(specified::LengthOrPercentage::parse(input));
                            try!(input.expect_comma());
                            let ty = try!(specified::LengthOrPercentage::parse(input));
                            try!(input.expect_comma());
                            let tz = try!(specified::Length::parse(input));
                            result.push(SpecifiedOperation::Translate(
                                TranslateKind::Translate3D,
                                tx,
                                ty,
                                tz));
                            Ok(())
                        }))

                    },
                    "scale" => {
                        try!(input.parse_nested_block(|input| {
                            let (sx, sy) = try!(parse_two_floats(input));
                            result.push(SpecifiedOperation::Scale(sx, sy, 1.0));
                            Ok(())
                        }))
                    },
                    "scalex" => {
                        try!(input.parse_nested_block(|input| {
                            let sx = try!(specified::parse_number(input));
                            result.push(SpecifiedOperation::Scale(sx, 1.0, 1.0));
                            Ok(())
                        }))
                    },
                    "scaley" => {
                        try!(input.parse_nested_block(|input| {
                            let sy = try!(specified::parse_number(input));
                            result.push(SpecifiedOperation::Scale(1.0, sy, 1.0));
                            Ok(())
                        }))
                    },
                    "scalez" => {
                        try!(input.parse_nested_block(|input| {
                            let sz = try!(specified::parse_number(input));
                            result.push(SpecifiedOperation::Scale(1.0, 1.0, sz));
                            Ok(())
                        }))
                    },
                    "scale3d" => {
                        try!(input.parse_nested_block(|input| {
                            let sx = try!(specified::parse_number(input));
                            try!(input.expect_comma());
                            let sy = try!(specified::parse_number(input));
                            try!(input.expect_comma());
                            let sz = try!(specified::parse_number(input));
                            result.push(SpecifiedOperation::Scale(sx, sy, sz));
                            Ok(())
                        }))
                    },
                    "rotate" => {
                        try!(input.parse_nested_block(|input| {
                            let theta = try!(specified::Angle::parse(input));
                            result.push(SpecifiedOperation::Rotate(0.0, 0.0, 1.0, theta));
                            Ok(())
                        }))
                    },
                    "rotatex" => {
                        try!(input.parse_nested_block(|input| {
                            let theta = try!(specified::Angle::parse(input));
                            result.push(SpecifiedOperation::Rotate(1.0, 0.0, 0.0, theta));
                            Ok(())
                        }))
                    },
                    "rotatey" => {
                        try!(input.parse_nested_block(|input| {
                            let theta = try!(specified::Angle::parse(input));
                            result.push(SpecifiedOperation::Rotate(0.0, 1.0, 0.0, theta));
                            Ok(())
                        }))
                    },
                    "rotatez" => {
                        try!(input.parse_nested_block(|input| {
                            let theta = try!(specified::Angle::parse(input));
                            result.push(SpecifiedOperation::Rotate(0.0, 0.0, 1.0, theta));
                            Ok(())
                        }))
                    },
                    "rotate3d" => {
                        try!(input.parse_nested_block(|input| {
                            let ax = try!(specified::parse_number(input));
                            try!(input.expect_comma());
                            let ay = try!(specified::parse_number(input));
                            try!(input.expect_comma());
                            let az = try!(specified::parse_number(input));
                            try!(input.expect_comma());
                            let theta = try!(specified::Angle::parse(input));
                            // TODO(gw): Check the axis can be normalized!!
                            result.push(SpecifiedOperation::Rotate(ax, ay, az, theta));
                            Ok(())
                        }))
                    },
                    "skew" => {
                        try!(input.parse_nested_block(|input| {
                            let (theta_x, theta_y) = try!(parse_two_angles(input));
                            result.push(SpecifiedOperation::Skew(theta_x, theta_y));
                            Ok(())
                        }))
                    },
                    "skewx" => {
                        try!(input.parse_nested_block(|input| {
                            let theta_x = try!(specified::Angle::parse(input));
                            result.push(SpecifiedOperation::Skew(theta_x, specified::Angle(0.0)));
                            Ok(())
                        }))
                    },
                    "skewy" => {
                        try!(input.parse_nested_block(|input| {
                            let theta_y = try!(specified::Angle::parse(input));
                            result.push(SpecifiedOperation::Skew(specified::Angle(0.0), theta_y));
                            Ok(())
                        }))
                    },
                    "perspective" => {
                        try!(input.parse_nested_block(|input| {
                            let d = try!(specified::Length::parse(input));
                            result.push(SpecifiedOperation::Perspective(d));
                            Ok(())
                        }))
                    },
                    _ => return Err(())
                }
            }

            if !result.is_empty() {
                Ok(SpecifiedValue(result))
            } else {
                Err(())
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                if self.0.is_empty() {
                    return computed_value::T(None)
                }

                let mut result = vec!();
                for operation in &self.0 {
                    match *operation {
                        SpecifiedOperation::Matrix(ref matrix) => {
                            result.push(computed_value::ComputedOperation::Matrix(*matrix));
                        }
                        SpecifiedOperation::Translate(_, ref tx, ref ty, ref tz) => {
                            result.push(computed_value::ComputedOperation::Translate(tx.to_computed_value(context),
                                                                                     ty.to_computed_value(context),
                                                                                     tz.to_computed_value(context)));
                        }
                        SpecifiedOperation::Scale(sx, sy, sz) => {
                            result.push(computed_value::ComputedOperation::Scale(sx, sy, sz));
                        }
                        SpecifiedOperation::Rotate(ax, ay, az, theta) => {
                            result.push(computed_value::ComputedOperation::Rotate(ax, ay, az, theta));
                        }
                        SpecifiedOperation::Skew(theta_x, theta_y) => {
                            result.push(computed_value::ComputedOperation::Skew(theta_x, theta_y));
                        }
                        SpecifiedOperation::Perspective(d) => {
                            result.push(computed_value::ComputedOperation::Perspective(d.to_computed_value(context)));
                        }
                    };
                }

                computed_value::T(Some(result))
            }
        }
    </%self:longhand>

    pub struct OriginParseResult {
        horizontal: Option<specified::LengthOrPercentage>,
        vertical: Option<specified::LengthOrPercentage>,
        depth: Option<specified::Length>
    }

    pub fn parse_origin(_: &ParserContext, input: &mut Parser) -> Result<OriginParseResult,()> {
        use values::specified::{LengthOrPercentage, Percentage};
        let (mut horizontal, mut vertical, mut depth) = (None, None, None);
        loop {
            if let Err(_) = input.try(|input| {
                let token = try!(input.expect_ident());
                match_ignore_ascii_case! {
                    token,
                    "left" => {
                        if horizontal.is_none() {
                            horizontal = Some(LengthOrPercentage::Percentage(Percentage(0.0)))
                        } else {
                            return Err(())
                        }
                    },
                    "center" => {
                        if horizontal.is_none() {
                            horizontal = Some(LengthOrPercentage::Percentage(Percentage(0.5)))
                        } else if vertical.is_none() {
                            vertical = Some(LengthOrPercentage::Percentage(Percentage(0.5)))
                        } else {
                            return Err(())
                        }
                    },
                    "right" => {
                        if horizontal.is_none() {
                            horizontal = Some(LengthOrPercentage::Percentage(Percentage(1.0)))
                        } else {
                            return Err(())
                        }
                    },
                    "top" => {
                        if vertical.is_none() {
                            vertical = Some(LengthOrPercentage::Percentage(Percentage(0.0)))
                        } else {
                            return Err(())
                        }
                    },
                    "bottom" => {
                        if vertical.is_none() {
                            vertical = Some(LengthOrPercentage::Percentage(Percentage(1.0)))
                        } else {
                            return Err(())
                        }
                    },
                    _ => return Err(())
                }
                Ok(())
            }) {
                match LengthOrPercentage::parse(input) {
                    Ok(value) => {
                        if horizontal.is_none() {
                            horizontal = Some(value);
                        } else if vertical.is_none() {
                            vertical = Some(value);
                        } else if let LengthOrPercentage::Length(length) = value {
                            depth = Some(length);
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        }

        if horizontal.is_some() || vertical.is_some() {
            Ok(OriginParseResult {
                horizontal: horizontal,
                vertical: vertical,
                depth: depth,
            })
        } else {
            Err(())
        }
    }

    ${single_keyword("backface-visibility", "visible hidden")}

    ${single_keyword("transform-style", "auto flat preserve-3d")}

    <%self:longhand name="transform-origin">
        use app_units::Au;
        use values::AuExtensionMethods;
        use values::specified::{Length, LengthOrPercentage, Percentage};

        use cssparser::ToCss;
        use std::fmt;

        pub mod computed_value {
            use values::computed::{Length, LengthOrPercentage};

            #[derive(Clone, Copy, Debug, PartialEq, HeapSizeOf)]
            pub struct T {
                pub horizontal: LengthOrPercentage,
                pub vertical: LengthOrPercentage,
                pub depth: Length,
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, HeapSizeOf)]
        pub struct SpecifiedValue {
            horizontal: LengthOrPercentage,
            vertical: LengthOrPercentage,
            depth: Length,
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.vertical.to_css(dest));
                try!(dest.write_str(" "));
                self.depth.to_css(dest)
            }
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.vertical.to_css(dest));
                try!(dest.write_str(" "));
                self.depth.to_css(dest)
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T {
                horizontal: computed::LengthOrPercentage::Percentage(0.5),
                vertical: computed::LengthOrPercentage::Percentage(0.5),
                depth: Au(0),
            }
        }

        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            let result = try!(super::parse_origin(context, input));
            Ok(SpecifiedValue {
                horizontal: result.horizontal.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.5))),
                vertical: result.vertical.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.5))),
                depth: result.depth.unwrap_or(Length::Absolute(Au(0))),
            })
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                computed_value::T {
                    horizontal: self.horizontal.to_computed_value(context),
                    vertical: self.vertical.to_computed_value(context),
                    depth: self.depth.to_computed_value(context),
                }
            }
        }
    </%self:longhand>

    ${predefined_type("perspective",
                      "LengthOrNone",
                      "computed::LengthOrNone::None")}

    <%self:longhand name="perspective-origin">
        use values::specified::{LengthOrPercentage, Percentage};

        use cssparser::ToCss;
        use std::fmt;

        pub mod computed_value {
            use values::computed::LengthOrPercentage;

            #[derive(Clone, Copy, Debug, PartialEq, HeapSizeOf)]
            pub struct T {
                pub horizontal: LengthOrPercentage,
                pub vertical: LengthOrPercentage,
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                self.vertical.to_css(dest)
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, HeapSizeOf)]
        pub struct SpecifiedValue {
            horizontal: LengthOrPercentage,
            vertical: LengthOrPercentage,
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                self.vertical.to_css(dest)
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T {
                horizontal: computed::LengthOrPercentage::Percentage(0.5),
                vertical: computed::LengthOrPercentage::Percentage(0.5),
            }
        }

        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            let result = try!(super::parse_origin(context, input));
            match result.depth {
                Some(_) => Err(()),
                None => Ok(SpecifiedValue {
                    horizontal: result.horizontal.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.5))),
                    vertical: result.vertical.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.5))),
                })
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
    </%self:longhand>

    ${single_keyword("mix-blend-mode",
                     """normal multiply screen overlay darken lighten color-dodge
                        color-burn hard-light soft-light difference exclusion hue
                        saturation color luminosity""")}

    ${switch_to_style_struct("InheritedBox")}

    <%self:longhand name="image-rendering">

        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;

            #[derive(Copy, Clone, Debug, PartialEq, HeapSizeOf, Deserialize, Serialize)]
            pub enum T {
                Auto,
                CrispEdges,
                Pixelated,
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        T::Auto => dest.write_str("auto"),
                        T::CrispEdges => dest.write_str("crisp-edges"),
                        T::Pixelated => dest.write_str("pixelated"),
                    }
                }
            }
        }

        pub type SpecifiedValue = computed_value::T;

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::Auto
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            // According to to CSS-IMAGES-3, `optimizespeed` and `optimizequality` are synonyms for
            // `auto`.
            match_ignore_ascii_case! {
                try!(input.expect_ident()),
                "auto" => Ok(computed_value::T::Auto),
                "optimizespeed" => Ok(computed_value::T::Auto),
                "optimizequality" => Ok(computed_value::T::Auto),
                "crisp-edges" => Ok(computed_value::T::CrispEdges),
                "pixelated" => Ok(computed_value::T::Pixelated),
                _ => Err(())
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, _: &Cx) -> computed_value::T {
                *self
            }
        }
    </%self:longhand>

    ${switch_to_style_struct("Box")}

    // TODO(pcwalton): Multiple transitions.
    <%self:longhand name="transition-duration">
        use values::specified::Time;

        pub use self::computed_value::T as SpecifiedValue;
        pub use values::specified::Time as SingleSpecifiedValue;

        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;
            use values::computed::{TContext, ToComputedValue};

            pub use values::computed::Time as SingleComputedValue;

            #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Vec<SingleComputedValue>);

            impl ToComputedValue for T {
                type ComputedValue = T;

                #[inline]
                fn to_computed_value<Cx: TContext>(&self, _: &Cx) -> T {
                    (*self).clone()
                }
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    if self.0.is_empty() {
                        return dest.write_str("none")
                    }
                    for (i, value) in self.0.iter().enumerate() {
                        if i != 0 {
                            try!(dest.write_str(", "))
                        }
                        try!(value.to_css(dest))
                    }
                    Ok(())
                }
            }
        }

        #[inline]
        pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue,()> {
            Time::parse(input)
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(vec![get_initial_single_value()])
        }

        #[inline]
        pub fn get_initial_single_value() -> Time {
            Time(0.0)
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
        }
    </%self:longhand>

    // TODO(pcwalton): Lots more timing functions.
    // TODO(pcwalton): Multiple transitions.
    <%self:longhand name="transition-timing-function">
        use self::computed_value::{StartEnd, TransitionTimingFunction};

        use euclid::point::Point2D;

        pub use self::computed_value::SingleComputedValue as SingleSpecifiedValue;
        pub use self::computed_value::T as SpecifiedValue;

        static EASE: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
            x: 0.25,
            y: 0.1,
        }, Point2D {
            x: 0.25,
            y: 1.0,
        });
        static LINEAR: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
            x: 0.0,
            y: 0.0,
        }, Point2D {
            x: 1.0,
            y: 1.0,
        });
        static EASE_IN: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
            x: 0.42,
            y: 0.0,
        }, Point2D {
            x: 1.0,
            y: 1.0,
        });
        static EASE_OUT: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
            x: 0.0,
            y: 0.0,
        }, Point2D {
            x: 0.58,
            y: 1.0,
        });
        static EASE_IN_OUT: TransitionTimingFunction =
            TransitionTimingFunction::CubicBezier(Point2D {
                x: 0.42,
                y: 0.0,
            }, Point2D {
                x: 0.58,
                y: 1.0,
            });
        static STEP_START: TransitionTimingFunction =
            TransitionTimingFunction::Steps(1, StartEnd::Start);
        static STEP_END: TransitionTimingFunction =
            TransitionTimingFunction::Steps(1, StartEnd::End);

        pub mod computed_value {
            use cssparser::ToCss;
            use euclid::point::Point2D;
            use std::fmt;

            pub use self::TransitionTimingFunction as SingleComputedValue;

            #[derive(Copy, Clone, Debug, PartialEq, HeapSizeOf)]
            pub enum TransitionTimingFunction {
                CubicBezier(Point2D<f32>, Point2D<f32>),
                Steps(u32, StartEnd),
            }

            impl ToCss for TransitionTimingFunction {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        TransitionTimingFunction::CubicBezier(p1, p2) => {
                            try!(dest.write_str("cubic-bezier("));
                            try!(p1.x.to_css(dest));
                            try!(dest.write_str(", "));
                            try!(p1.y.to_css(dest));
                            try!(dest.write_str(", "));
                            try!(p2.x.to_css(dest));
                            try!(dest.write_str(", "));
                            try!(p2.y.to_css(dest));
                            dest.write_str(")")
                        }
                        TransitionTimingFunction::Steps(steps, start_end) => {
                            try!(dest.write_str("steps("));
                            try!(steps.to_css(dest));
                            try!(dest.write_str(", "));
                            try!(start_end.to_css(dest));
                            dest.write_str(")")
                        }
                    }
                }
            }

            #[derive(Copy, Clone, Debug, PartialEq, HeapSizeOf)]
            pub enum StartEnd {
                Start,
                End,
            }

            impl ToCss for StartEnd {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        StartEnd::Start => dest.write_str("start"),
                        StartEnd::End => dest.write_str("end"),
                    }
                }
            }

            #[derive(Clone, Debug, PartialEq, HeapSizeOf)]
            pub struct T(pub Vec<TransitionTimingFunction>);

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    if self.0.is_empty() {
                        return dest.write_str("none")
                    }
                    for (i, value) in self.0.iter().enumerate() {
                        if i != 0 {
                            try!(dest.write_str(", "))
                        }
                        try!(value.to_css(dest))
                    }
                    Ok(())
                }
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, _: &Cx) -> computed_value::T {
                (*self).clone()
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(vec![get_initial_single_value()])
        }

        #[inline]
        pub fn get_initial_single_value() -> TransitionTimingFunction {
            EASE
        }

        pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue,()> {
            if let Ok(function_name) = input.try(|input| input.expect_function()) {
                return match_ignore_ascii_case! {
                    function_name,
                    "cubic-bezier" => {
                        let (mut p1x, mut p1y, mut p2x, mut p2y) = (0.0, 0.0, 0.0, 0.0);
                        try!(input.parse_nested_block(|input| {
                            p1x = try!(specified::parse_number(input));
                            try!(input.expect_comma());
                            p1y = try!(specified::parse_number(input));
                            try!(input.expect_comma());
                            p2x = try!(specified::parse_number(input));
                            try!(input.expect_comma());
                            p2y = try!(specified::parse_number(input));
                            Ok(())
                        }));
                        let (p1, p2) = (Point2D::new(p1x, p1y), Point2D::new(p2x, p2y));
                        Ok(TransitionTimingFunction::CubicBezier(p1, p2))
                    },
                    "steps" => {
                        let (mut step_count, mut start_end) = (0, computed_value::StartEnd::Start);
                        try!(input.parse_nested_block(|input| {
                            step_count = try!(specified::parse_integer(input));
                            try!(input.expect_comma());
                            start_end = try!(match_ignore_ascii_case! {
                                try!(input.expect_ident()),
                                "start" => Ok(computed_value::StartEnd::Start),
                                "end" => Ok(computed_value::StartEnd::End),
                                _ => Err(())
                            });
                            Ok(())
                        }));
                        Ok(TransitionTimingFunction::Steps(step_count as u32, start_end))
                    },
                    _ => Err(())
                }
            }
            match_ignore_ascii_case! {
                try!(input.expect_ident()),
                "ease" => Ok(EASE),
                "linear" => Ok(LINEAR),
                "ease-in" => Ok(EASE_IN),
                "ease-out" => Ok(EASE_OUT),
                "ease-in-out" => Ok(EASE_IN_OUT),
                "step-start" => Ok(STEP_START),
                "step-end" => Ok(STEP_END),
                _ => Err(())
            }
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
        }
    </%self:longhand>

    // TODO(pcwalton): Lots more properties.
    <%self:longhand name="transition-property">
        use self::computed_value::TransitionProperty;

        pub use self::computed_value::SingleComputedValue as SingleSpecifiedValue;
        pub use self::computed_value::T as SpecifiedValue;

        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;

            pub use self::TransitionProperty as SingleComputedValue;

            #[derive(Copy, Clone, Debug, PartialEq, HeapSizeOf)]
            pub enum TransitionProperty {
                All,
                BackgroundColor,
                BackgroundPosition,
                BorderBottomColor,
                BorderBottomWidth,
                BorderLeftColor,
                BorderLeftWidth,
                BorderRightColor,
                BorderRightWidth,
                BorderSpacing,
                BorderTopColor,
                BorderTopWidth,
                Bottom,
                Color,
                Clip,
                FontSize,
                FontWeight,
                Height,
                Left,
                LetterSpacing,
                LineHeight,
                MarginBottom,
                MarginLeft,
                MarginRight,
                MarginTop,
                MaxHeight,
                MaxWidth,
                MinHeight,
                MinWidth,
                Opacity,
                OutlineColor,
                OutlineWidth,
                PaddingBottom,
                PaddingLeft,
                PaddingRight,
                PaddingTop,
                Right,
                TextIndent,
                TextShadow,
                Top,
                Transform,
                VerticalAlign,
                Visibility,
                Width,
                WordSpacing,
                ZIndex,
            }

            pub static ALL_TRANSITION_PROPERTIES: [TransitionProperty; 45] = [
                TransitionProperty::BackgroundColor,
                TransitionProperty::BackgroundPosition,
                TransitionProperty::BorderBottomColor,
                TransitionProperty::BorderBottomWidth,
                TransitionProperty::BorderLeftColor,
                TransitionProperty::BorderLeftWidth,
                TransitionProperty::BorderRightColor,
                TransitionProperty::BorderRightWidth,
                TransitionProperty::BorderSpacing,
                TransitionProperty::BorderTopColor,
                TransitionProperty::BorderTopWidth,
                TransitionProperty::Bottom,
                TransitionProperty::Color,
                TransitionProperty::Clip,
                TransitionProperty::FontSize,
                TransitionProperty::FontWeight,
                TransitionProperty::Height,
                TransitionProperty::Left,
                TransitionProperty::LetterSpacing,
                TransitionProperty::LineHeight,
                TransitionProperty::MarginBottom,
                TransitionProperty::MarginLeft,
                TransitionProperty::MarginRight,
                TransitionProperty::MarginTop,
                TransitionProperty::MaxHeight,
                TransitionProperty::MaxWidth,
                TransitionProperty::MinHeight,
                TransitionProperty::MinWidth,
                TransitionProperty::Opacity,
                TransitionProperty::OutlineColor,
                TransitionProperty::OutlineWidth,
                TransitionProperty::PaddingBottom,
                TransitionProperty::PaddingLeft,
                TransitionProperty::PaddingRight,
                TransitionProperty::PaddingTop,
                TransitionProperty::Right,
                TransitionProperty::TextIndent,
                TransitionProperty::TextShadow,
                TransitionProperty::Top,
                TransitionProperty::Transform,
                TransitionProperty::VerticalAlign,
                TransitionProperty::Visibility,
                TransitionProperty::Width,
                TransitionProperty::WordSpacing,
                TransitionProperty::ZIndex,
            ];

            impl ToCss for TransitionProperty {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        TransitionProperty::All => dest.write_str("all"),
                        TransitionProperty::BackgroundColor => dest.write_str("background-color"),
                        TransitionProperty::BackgroundPosition => dest.write_str("background-position"),
                        TransitionProperty::BorderBottomColor => dest.write_str("border-bottom-color"),
                        TransitionProperty::BorderBottomWidth => dest.write_str("border-bottom-width"),
                        TransitionProperty::BorderLeftColor => dest.write_str("border-left-color"),
                        TransitionProperty::BorderLeftWidth => dest.write_str("border-left-width"),
                        TransitionProperty::BorderRightColor => dest.write_str("border-right-color"),
                        TransitionProperty::BorderRightWidth => dest.write_str("border-right-width"),
                        TransitionProperty::BorderSpacing => dest.write_str("border-spacing"),
                        TransitionProperty::BorderTopColor => dest.write_str("border-top-color"),
                        TransitionProperty::BorderTopWidth => dest.write_str("border-top-width"),
                        TransitionProperty::Bottom => dest.write_str("bottom"),
                        TransitionProperty::Color => dest.write_str("color"),
                        TransitionProperty::Clip => dest.write_str("clip"),
                        TransitionProperty::FontSize => dest.write_str("font-size"),
                        TransitionProperty::FontWeight => dest.write_str("font-weight"),
                        TransitionProperty::Height => dest.write_str("height"),
                        TransitionProperty::Left => dest.write_str("left"),
                        TransitionProperty::LetterSpacing => dest.write_str("letter-spacing"),
                        TransitionProperty::LineHeight => dest.write_str("line-height"),
                        TransitionProperty::MarginBottom => dest.write_str("margin-bottom"),
                        TransitionProperty::MarginLeft => dest.write_str("margin-left"),
                        TransitionProperty::MarginRight => dest.write_str("margin-right"),
                        TransitionProperty::MarginTop => dest.write_str("margin-top"),
                        TransitionProperty::MaxHeight => dest.write_str("max-height"),
                        TransitionProperty::MaxWidth => dest.write_str("max-width"),
                        TransitionProperty::MinHeight => dest.write_str("min-height"),
                        TransitionProperty::MinWidth => dest.write_str("min-width"),
                        TransitionProperty::Opacity => dest.write_str("opacity"),
                        TransitionProperty::OutlineColor => dest.write_str("outline-color"),
                        TransitionProperty::OutlineWidth => dest.write_str("outline-width"),
                        TransitionProperty::PaddingBottom => dest.write_str("padding-bottom"),
                        TransitionProperty::PaddingLeft => dest.write_str("padding-left"),
                        TransitionProperty::PaddingRight => dest.write_str("padding-right"),
                        TransitionProperty::PaddingTop => dest.write_str("padding-top"),
                        TransitionProperty::Right => dest.write_str("right"),
                        TransitionProperty::TextIndent => dest.write_str("text-indent"),
                        TransitionProperty::TextShadow => dest.write_str("text-shadow"),
                        TransitionProperty::Top => dest.write_str("top"),
                        TransitionProperty::Transform => dest.write_str("transform"),
                        TransitionProperty::VerticalAlign => dest.write_str("vertical-align"),
                        TransitionProperty::Visibility => dest.write_str("visibility"),
                        TransitionProperty::Width => dest.write_str("width"),
                        TransitionProperty::WordSpacing => dest.write_str("word-spacing"),
                        TransitionProperty::ZIndex => dest.write_str("z-index"),
                    }
                }
            }

            #[derive(Clone, Debug, PartialEq, HeapSizeOf)]
            pub struct T(pub Vec<SingleComputedValue>);

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    if self.0.is_empty() {
                        return dest.write_str("none")
                    }
                    for (i, value) in self.0.iter().enumerate() {
                        if i != 0 {
                            try!(dest.write_str(", "))
                        }
                        try!(value.to_css(dest))
                    }
                    Ok(())
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(Vec::new())
        }

        pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue,()> {
            match_ignore_ascii_case! {
                try!(input.expect_ident()),
                "all" => Ok(TransitionProperty::All),
                "background-color" => Ok(TransitionProperty::BackgroundColor),
                "background-position" => Ok(TransitionProperty::BackgroundPosition),
                "border-bottom-color" => Ok(TransitionProperty::BorderBottomColor),
                "border-bottom-width" => Ok(TransitionProperty::BorderBottomWidth),
                "border-left-color" => Ok(TransitionProperty::BorderLeftColor),
                "border-left-width" => Ok(TransitionProperty::BorderLeftWidth),
                "border-right-color" => Ok(TransitionProperty::BorderRightColor),
                "border-right-width" => Ok(TransitionProperty::BorderRightWidth),
                "border-spacing" => Ok(TransitionProperty::BorderSpacing),
                "border-top-color" => Ok(TransitionProperty::BorderTopColor),
                "border-top-width" => Ok(TransitionProperty::BorderTopWidth),
                "bottom" => Ok(TransitionProperty::Bottom),
                "color" => Ok(TransitionProperty::Color),
                "clip" => Ok(TransitionProperty::Clip),
                "font-size" => Ok(TransitionProperty::FontSize),
                "font-weight" => Ok(TransitionProperty::FontWeight),
                "height" => Ok(TransitionProperty::Height),
                "left" => Ok(TransitionProperty::Left),
                "letter-spacing" => Ok(TransitionProperty::LetterSpacing),
                "line-height" => Ok(TransitionProperty::LineHeight),
                "margin-bottom" => Ok(TransitionProperty::MarginBottom),
                "margin-left" => Ok(TransitionProperty::MarginLeft),
                "margin-right" => Ok(TransitionProperty::MarginRight),
                "margin-top" => Ok(TransitionProperty::MarginTop),
                "max-height" => Ok(TransitionProperty::MaxHeight),
                "max-width" => Ok(TransitionProperty::MaxWidth),
                "min-height" => Ok(TransitionProperty::MinHeight),
                "min-width" => Ok(TransitionProperty::MinWidth),
                "opacity" => Ok(TransitionProperty::Opacity),
                "outline-color" => Ok(TransitionProperty::OutlineColor),
                "outline-width" => Ok(TransitionProperty::OutlineWidth),
                "padding-bottom" => Ok(TransitionProperty::PaddingBottom),
                "padding-left" => Ok(TransitionProperty::PaddingLeft),
                "padding-right" => Ok(TransitionProperty::PaddingRight),
                "padding-top" => Ok(TransitionProperty::PaddingTop),
                "right" => Ok(TransitionProperty::Right),
                "text-indent" => Ok(TransitionProperty::TextIndent),
                "text-shadow" => Ok(TransitionProperty::TextShadow),
                "top" => Ok(TransitionProperty::Top),
                "transform" => Ok(TransitionProperty::Transform),
                "vertical-align" => Ok(TransitionProperty::VerticalAlign),
                "visibility" => Ok(TransitionProperty::Visibility),
                "width" => Ok(TransitionProperty::Width),
                "word-spacing" => Ok(TransitionProperty::WordSpacing),
                "z-index" => Ok(TransitionProperty::ZIndex),
                _ => Err(())
            }
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, _: &Cx) -> computed_value::T {
                (*self).clone()
            }
        }
    </%self:longhand>

    <%self:longhand name="transition-delay">
        pub use properties::longhands::transition_duration::{SingleSpecifiedValue, SpecifiedValue};
        pub use properties::longhands::transition_duration::{computed_value};
        pub use properties::longhands::transition_duration::{get_initial_single_value};
        pub use properties::longhands::transition_duration::{get_initial_value, parse, parse_one};
    </%self:longhand>

    // CSS Flexible Box Layout Module Level 1
    // http://www.w3.org/TR/css3-flexbox/

    ${switch_to_style_struct("Position")}

    // Flex container properties
    ${single_keyword("flex-direction", "row row-reverse column column-reverse", experimental=True)}
}


pub mod shorthands {
    use cssparser::Parser;
    use parser::ParserContext;
    use values::specified;

    <%def name="shorthand(name, sub_properties, experimental=False)">
    <%
        shorthand = Shorthand(name, sub_properties.split(), experimental=experimental)
        SHORTHANDS.append(shorthand)
    %>
        pub mod ${shorthand.ident} {
            use cssparser::Parser;
            use parser::ParserContext;
            use properties::{longhands, PropertyDeclaration, DeclaredValue, Shorthand};

            pub struct Longhands {
                % for sub_property in shorthand.sub_properties:
                    pub ${sub_property.ident}:
                        Option<longhands::${sub_property.ident}::SpecifiedValue>,
                % endfor
            }

            pub fn parse(context: &ParserContext, input: &mut Parser,
                         declarations: &mut Vec<PropertyDeclaration>)
                         -> Result<(), ()> {
                input.look_for_var_functions();
                let start = input.position();
                let value = input.parse_entirely(|input| parse_value(context, input));
                if value.is_err() {
                    while let Ok(_) = input.next() {}  // Look for var() after the error.
                }
                let var = input.seen_var_functions();
                if let Ok(value) = value {
                    % for sub_property in shorthand.sub_properties:
                        declarations.push(PropertyDeclaration::${sub_property.camel_case}(
                            match value.${sub_property.ident} {
                                Some(value) => DeclaredValue::Value(value),
                                None => DeclaredValue::Initial,
                            }
                        ));
                    % endfor
                    Ok(())
                } else if var {
                    input.reset(start);
                    let (first_token_type, css) = try!(
                        ::custom_properties::parse_non_custom_with_var(input));
                    % for sub_property in shorthand.sub_properties:
                        declarations.push(PropertyDeclaration::${sub_property.camel_case}(
                            DeclaredValue::WithVariables {
                                css: css.clone().into_owned(),
                                first_token_type: first_token_type,
                                base_url: context.base_url.clone(),
                                from_shorthand: Some(Shorthand::${shorthand.camel_case}),
                            }
                        ));
                    % endfor
                    Ok(())
                } else {
                    Err(())
                }
            }

            #[allow(unused_variables)]
            pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
                ${caller.body()}
            }
        }
    </%def>

    fn parse_four_sides<F, T>(input: &mut Parser, parse_one: F) -> Result<(T, T, T, T), ()>
    where F: Fn(&mut Parser) -> Result<T, ()>, F: Copy, T: Clone {
        // zero or more than four values is invalid.
        // one value sets them all
        // two values set (top, bottom) and (left, right)
        // three values set top, (left, right) and bottom
        // four values set them in order
        let top = try!(parse_one(input));
        let right;
        let bottom;
        let left;
        match input.try(parse_one) {
            Err(()) => {
                right = top.clone();
                bottom = top.clone();
                left = top.clone();
            }
            Ok(value) => {
                right = value;
                match input.try(parse_one) {
                    Err(()) => {
                        bottom = top.clone();
                        left = right.clone();
                    }
                    Ok(value) => {
                        bottom = value;
                        match input.try(parse_one) {
                            Err(()) => {
                                left = right.clone();
                            }
                            Ok(value) => {
                                left = value;
                            }
                        }

                    }
                }

            }
        }
        Ok((top, right, bottom, left))
    }

    <%def name="four_sides_shorthand(name, sub_property_pattern, parser_function)">
        <%self:shorthand name="${name}" sub_properties="${
                ' '.join(sub_property_pattern % side
                         for side in ['top', 'right', 'bottom', 'left'])}">
            use super::parse_four_sides;
            use values::specified;
            let _unused = context;
            let (top, right, bottom, left) = try!(parse_four_sides(input, ${parser_function}));
            Ok(Longhands {
                % for side in ["top", "right", "bottom", "left"]:
                    ${to_rust_ident(sub_property_pattern % side)}: Some(${side}),
                % endfor
            })
        </%self:shorthand>
    </%def>

    // TODO: other background-* properties
    <%self:shorthand name="background"
                     sub_properties="background-color background-position background-repeat background-attachment
                                     background-image background-size background-origin background-clip">
        use properties::longhands::{background_color, background_position, background_repeat, background_attachment};
        use properties::longhands::{background_image, background_size, background_origin, background_clip};

        let mut color = None;
        let mut image = None;
        let mut position = None;
        let mut repeat = None;
        let mut size = None;
        let mut attachment = None;
        let mut any = false;
        let mut origin = None;
        let mut clip = None;

        loop {
            if position.is_none() {
                if let Ok(value) = input.try(|input| background_position::parse(context, input)) {
                    position = Some(value);
                    any = true;

                    // Parse background size, if applicable.
                    size = input.try(|input| {
                        try!(input.expect_delim('/'));
                        background_size::parse(context, input)
                    }).ok();

                    continue
                }
            }
            if color.is_none() {
                if let Ok(value) = input.try(|input| background_color::parse(context, input)) {
                    color = Some(value);
                    any = true;
                    continue
                }
            }
            if image.is_none() {
                if let Ok(value) = input.try(|input| background_image::parse(context, input)) {
                    image = Some(value);
                    any = true;
                    continue
                }
            }
            if repeat.is_none() {
                if let Ok(value) = input.try(|input| background_repeat::parse(context, input)) {
                    repeat = Some(value);
                    any = true;
                    continue
                }
            }
            if attachment.is_none() {
                if let Ok(value) = input.try(|input| background_attachment::parse(context, input)) {
                    attachment = Some(value);
                    any = true;
                    continue
                }
            }
            if origin.is_none() {
                if let Ok(value) = input.try(|input| background_origin::parse(context, input)) {
                    origin = Some(value);
                    any = true;
                    continue
                }
            }
            if clip.is_none() {
                if let Ok(value) = input.try(|input| background_clip::parse(context, input)) {
                    clip = Some(value);
                    any = true;
                    continue
                }
            }
            break
        }

        if any {
            Ok(Longhands {
                background_color: color,
                background_image: image,
                background_position: position,
                background_repeat: repeat,
                background_attachment: attachment,
                background_size: size,
                background_origin: origin,
                background_clip: clip,
            })
        } else {
            Err(())
        }
    </%self:shorthand>

    ${four_sides_shorthand("margin", "margin-%s", "specified::LengthOrPercentageOrAuto::parse")}
    ${four_sides_shorthand("padding", "padding-%s", "specified::LengthOrPercentage::parse")}

    ${four_sides_shorthand("border-color", "border-%s-color", "specified::CSSColor::parse")}
    ${four_sides_shorthand("border-style", "border-%s-style",
                           "specified::BorderStyle::parse")}
    <%self:shorthand name="border-width" sub_properties="${
            ' '.join('border-%s-width' % side
                     for side in ['top', 'right', 'bottom', 'left'])}">
        use super::parse_four_sides;
        use values::specified;
        let _unused = context;
        let (top, right, bottom, left) = try!(parse_four_sides(input, specified::parse_border_width));
        Ok(Longhands {
            % for side in ["top", "right", "bottom", "left"]:
                ${to_rust_ident('border-%s-width' % side)}:
                    Some(longhands::${to_rust_ident('border-%s-width' % side)}::SpecifiedValue(${side})),
            % endfor
        })
    </%self:shorthand>


    pub fn parse_border(context: &ParserContext, input: &mut Parser)
                     -> Result<(Option<specified::CSSColor>,
                                Option<specified::BorderStyle>,
                                Option<specified::Length>), ()> {
        use values::specified;
        let _unused = context;
        let mut color = None;
        let mut style = None;
        let mut width = None;
        let mut any = false;
        loop {
            if color.is_none() {
                if let Ok(value) = input.try(specified::CSSColor::parse) {
                    color = Some(value);
                    any = true;
                    continue
                }
            }
            if style.is_none() {
                if let Ok(value) = input.try(specified::BorderStyle::parse) {
                    style = Some(value);
                    any = true;
                    continue
                }
            }
            if width.is_none() {
                if let Ok(value) = input.try(specified::parse_border_width) {
                    width = Some(value);
                    any = true;
                    continue
                }
            }
            break
        }
        if any { Ok((color, style, width)) } else { Err(()) }
    }


    % for side in ["top", "right", "bottom", "left"]:
        <%self:shorthand name="border-${side}" sub_properties="${' '.join(
            'border-%s-%s' % (side, prop)
            for prop in ['color', 'style', 'width']
        )}">
            let (color, style, width) = try!(super::parse_border(context, input));
            Ok(Longhands {
                border_${side}_color: color,
                border_${side}_style: style,
                border_${side}_width:
                    width.map(longhands::${to_rust_ident('border-%s-width' % side)}::SpecifiedValue),
            })
        </%self:shorthand>
    % endfor

    <%self:shorthand name="border" sub_properties="${' '.join(
        'border-%s-%s' % (side, prop)
        for side in ['top', 'right', 'bottom', 'left']
        for prop in ['color', 'style', 'width']
    )}">
        let (color, style, width) = try!(super::parse_border(context, input));
        Ok(Longhands {
            % for side in ["top", "right", "bottom", "left"]:
                border_${side}_color: color.clone(),
                border_${side}_style: style,
                border_${side}_width:
                    width.map(longhands::${to_rust_ident('border-%s-width' % side)}::SpecifiedValue),
            % endfor
        })
    </%self:shorthand>

    <%self:shorthand name="border-radius" sub_properties="${' '.join(
        'border-%s-radius' % (corner)
         for corner in ['top-left', 'top-right', 'bottom-right', 'bottom-left']
    )}">
        use app_units::Au;
        use values::specified::{Length, LengthOrPercentage};
        use values::specified::BorderRadiusSize;

        let _ignored = context;

        fn parse_one_set_of_border_values(mut input: &mut Parser)
                                         -> Result<[LengthOrPercentage; 4], ()> {
            let mut count = 0;
            let mut values = [LengthOrPercentage::Length(Length::Absolute(Au(0))); 4];
            while count < 4 {
                if let Ok(value) = input.try(LengthOrPercentage::parse) {
                    values[count] = value;
                    count += 1;
                } else {
                    break
                }
            }

            match count {
                1 => Ok([values[0], values[0], values[0], values[0]]),
                2 => Ok([values[0], values[1], values[0], values[1]]),
                3 => Ok([values[0], values[1], values[2], values[1]]),
                4 => Ok([values[0], values[1], values[2], values[3]]),
                _ => Err(()),
            }
        }

        fn parse_one_set_of_border_radii(mut input: &mut Parser)
                                         -> Result<[BorderRadiusSize; 4], ()> {
            let widths = try!(parse_one_set_of_border_values(input));
            let mut heights = widths.clone();
            let mut radii_values = [BorderRadiusSize::zero(); 4];
            if input.try(|input| input.expect_delim('/')).is_ok() {
                heights = try!(parse_one_set_of_border_values(input));
            }
            for i in 0..radii_values.len() {
                radii_values[i] = BorderRadiusSize::new(widths[i], heights[i]);
            }
            Ok(radii_values)
        }

        let radii = try!(parse_one_set_of_border_radii(input));
        Ok(Longhands {
            border_top_left_radius: Some(radii[0]),
            border_top_right_radius: Some(radii[1]),
            border_bottom_right_radius: Some(radii[2]),
            border_bottom_left_radius: Some(radii[3]),
        })
    </%self:shorthand>

    <%self:shorthand name="outline" sub_properties="outline-color outline-style outline-width">
        use properties::longhands::outline_width;
        use values::specified;

        let _unused = context;
        let mut color = None;
        let mut style = None;
        let mut width = None;
        let mut any = false;
        loop {
            if color.is_none() {
                if let Ok(value) = input.try(specified::CSSColor::parse) {
                    color = Some(value);
                    any = true;
                    continue
                }
            }
            if style.is_none() {
                if let Ok(value) = input.try(specified::BorderStyle::parse) {
                    style = Some(value);
                    any = true;
                    continue
                }
            }
            if width.is_none() {
                if let Ok(value) = input.try(|input| outline_width::parse(context, input)) {
                    width = Some(value);
                    any = true;
                    continue
                }
            }
            break
        }
        if any {
            Ok(Longhands {
                outline_color: color,
                outline_style: style,
                outline_width: width,
            })
        } else {
            Err(())
        }
    </%self:shorthand>

    <%self:shorthand name="font" sub_properties="font-style font-variant font-weight
                                                 font-size line-height font-family">
        use properties::longhands::{font_style, font_variant, font_weight, font_size,
                                    line_height, font_family};
        let mut nb_normals = 0;
        let mut style = None;
        let mut variant = None;
        let mut weight = None;
        let size;
        loop {
            // Special-case 'normal' because it is valid in each of
            // font-style, font-weight and font-variant.
            // Leaves the values to None, 'normal' is the initial value for each of them.
            if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                nb_normals += 1;
                continue;
            }
            if style.is_none() {
                if let Ok(value) = input.try(|input| font_style::parse(context, input)) {
                    style = Some(value);
                    continue
                }
            }
            if weight.is_none() {
                if let Ok(value) = input.try(|input| font_weight::parse(context, input)) {
                    weight = Some(value);
                    continue
                }
            }
            if variant.is_none() {
                if let Ok(value) = input.try(|input| font_variant::parse(context, input)) {
                    variant = Some(value);
                    continue
                }
            }
            size = Some(try!(font_size::parse(context, input)));
            break
        }
        #[inline]
        fn count<T>(opt: &Option<T>) -> u8 {
            if opt.is_some() { 1 } else { 0 }
        }
        if size.is_none() || (count(&style) + count(&weight) + count(&variant) + nb_normals) > 3 {
            return Err(())
        }
        let line_height = if input.try(|input| input.expect_delim('/')).is_ok() {
            Some(try!(line_height::parse(context, input)))
        } else {
            None
        };
        let family = try!(input.parse_comma_separated(font_family::parse_one_family));
        Ok(Longhands {
            font_style: style,
            font_variant: variant,
            font_weight: weight,
            font_size: size,
            line_height: line_height,
            font_family: Some(font_family::SpecifiedValue(family))
        })
    </%self:shorthand>

    // Per CSS-TEXT 6.2, "for legacy reasons, UAs must treat `word-wrap` as an alternate name for
    // the `overflow-wrap` property, as if it were a shorthand of `overflow-wrap`."
    <%self:shorthand name="word-wrap" sub_properties="overflow-wrap">
        use properties::longhands::overflow_wrap;
        Ok(Longhands {
            overflow_wrap: Some(try!(overflow_wrap::parse(context, input))),
        })
    </%self:shorthand>

    <%self:shorthand name="list-style"
                     sub_properties="list-style-image list-style-position list-style-type">
        use properties::longhands::{list_style_image, list_style_position, list_style_type};

        // `none` is ambiguous until we've finished parsing the shorthands, so we count the number
        // of times we see it.
        let mut nones = 0u8;
        let (mut image, mut position, mut list_style_type, mut any) = (None, None, None, false);
        loop {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                nones = nones + 1;
                if nones > 2 {
                    return Err(())
                }
                any = true;
                continue
            }

            if list_style_type.is_none() {
                if let Ok(value) = input.try(|input| list_style_type::parse(context, input)) {
                    list_style_type = Some(value);
                    any = true;
                    continue
                }
            }

            if image.is_none() {
                if let Ok(value) = input.try(|input| list_style_image::parse(context, input)) {
                    image = Some(value);
                    any = true;
                    continue
                }
            }

            if position.is_none() {
                if let Ok(value) = input.try(|input| list_style_position::parse(context, input)) {
                    position = Some(value);
                    any = true;
                    continue
                }
            }
            break
        }

        // If there are two `none`s, then we can't have a type or image; if there is one `none`,
        // then we can't have both a type *and* an image; if there is no `none` then we're fine as
        // long as we parsed something.
        match (any, nones, list_style_type, image) {
            (true, 2, None, None) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(list_style_image::SpecifiedValue::None),
                    list_style_type: Some(list_style_type::SpecifiedValue::none),
                })
            }
            (true, 1, None, Some(image)) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(image),
                    list_style_type: Some(list_style_type::SpecifiedValue::none),
                })
            }
            (true, 1, Some(list_style_type), None) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(list_style_image::SpecifiedValue::None),
                    list_style_type: Some(list_style_type),
                })
            }
            (true, 1, None, None) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(list_style_image::SpecifiedValue::None),
                    list_style_type: Some(list_style_type::SpecifiedValue::none),
                })
            }
            (true, 0, list_style_type, image) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: image,
                    list_style_type: list_style_type,
                })
            }
            _ => Err(()),
        }
    </%self:shorthand>

    <%self:shorthand name="columns" sub_properties="column-count column-width" experimental="True">
        use properties::longhands::{column_count, column_width};
        let mut column_count = None;
        let mut column_width = None;
        let mut autos = 0;

        loop {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                // Leave the options to None, 'auto' is the initial value.
                autos += 1;
                continue
            }

            if column_count.is_none() {
                if let Ok(value) = input.try(|input| column_count::parse(context, input)) {
                    column_count = Some(value);
                    continue
                }
            }

            if column_width.is_none() {
                if let Ok(value) = input.try(|input| column_width::parse(context, input)) {
                    column_width = Some(value);
                    continue
                }
            }

            break
        }

        let values = autos + column_count.iter().len() + column_width.iter().len();
        if values == 0 || values > 2 {
            Err(())
        } else {
            Ok(Longhands {
                column_count: column_count,
                column_width: column_width,
            })
        }
    </%self:shorthand>

    <%self:shorthand name="overflow" sub_properties="overflow-x overflow-y">
        use properties::longhands::{overflow_x, overflow_y};

        let overflow = try!(overflow_x::parse(context, input));
        Ok(Longhands {
            overflow_x: Some(overflow),
            overflow_y: Some(overflow_y::SpecifiedValue(overflow)),
        })
    </%self:shorthand>

    <%self:shorthand name="transition"
                     sub_properties="transition-property transition-duration transition-timing-function
                                     transition-delay">
        use properties::longhands::{transition_delay, transition_duration, transition_property};
        use properties::longhands::{transition_timing_function};

        struct SingleTransition {
            transition_property: transition_property::SingleSpecifiedValue,
            transition_duration: transition_duration::SingleSpecifiedValue,
            transition_timing_function: transition_timing_function::SingleSpecifiedValue,
            transition_delay: transition_delay::SingleSpecifiedValue,
        }

        fn parse_one_transition(input: &mut Parser) -> Result<SingleTransition,()> {
            let (mut property, mut duration) = (None, None);
            let (mut timing_function, mut delay) = (None, None);
            loop {
                if property.is_none() {
                    if let Ok(value) = input.try(|input| transition_property::parse_one(input)) {
                        property = Some(value);
                        continue
                    }
                }

                if duration.is_none() {
                    if let Ok(value) = input.try(|input| transition_duration::parse_one(input)) {
                        duration = Some(value);
                        continue
                    }
                }

                if timing_function.is_none() {
                    if let Ok(value) = input.try(|input| {
                        transition_timing_function::parse_one(input)
                    }) {
                        timing_function = Some(value);
                        continue
                    }
                }

                if delay.is_none() {
                    if let Ok(value) = input.try(|input| transition_delay::parse_one(input)) {
                        delay = Some(value);
                        continue;
                    }
                }

                break
            }

            if let Some(property) = property {
                Ok(SingleTransition {
                    transition_property: property,
                    transition_duration:
                        duration.unwrap_or(transition_duration::get_initial_single_value()),
                    transition_timing_function:
                        timing_function.unwrap_or(
                            transition_timing_function::get_initial_single_value()),
                    transition_delay:
                        delay.unwrap_or(transition_delay::get_initial_single_value()),
                })
            } else {
                Err(())
            }
        }

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(Longhands {
                transition_property: None,
                transition_duration: None,
                transition_timing_function: None,
                transition_delay: None,
            })
        }

        let results = try!(input.parse_comma_separated(parse_one_transition));
        let (mut properties, mut durations) = (Vec::new(), Vec::new());
        let (mut timing_functions, mut delays) = (Vec::new(), Vec::new());
        for result in results {
            properties.push(result.transition_property);
            durations.push(result.transition_duration);
            timing_functions.push(result.transition_timing_function);
            delays.push(result.transition_delay);
        }

        Ok(Longhands {
            transition_property: Some(transition_property::SpecifiedValue(properties)),
            transition_duration: Some(transition_duration::SpecifiedValue(durations)),
            transition_timing_function:
                Some(transition_timing_function::SpecifiedValue(timing_functions)),
            transition_delay: Some(transition_delay::SpecifiedValue(delays)),
        })
    </%self:shorthand>
}


// TODO(SimonSapin): Convert this to a syntax extension rather than a Mako template.
// Maybe submit for inclusion in libstd?
mod property_bit_field {

    pub struct PropertyBitField {
        storage: [u32; (${len(LONGHANDS)} - 1 + 32) / 32]
    }

    impl PropertyBitField {
        #[inline]
        pub fn new() -> PropertyBitField {
            PropertyBitField { storage: [0; (${len(LONGHANDS)} - 1 + 32) / 32] }
        }

        #[inline]
        fn get(&self, bit: usize) -> bool {
            (self.storage[bit / 32] & (1 << (bit % 32))) != 0
        }
        #[inline]
        fn set(&mut self, bit: usize) {
            self.storage[bit / 32] |= 1 << (bit % 32)
        }
        % for i, property in enumerate(LONGHANDS):
            % if property.derived_from is None:
                #[allow(non_snake_case)]
                #[inline]
                pub fn get_${property.ident}(&self) -> bool {
                    self.get(${i})
                }
                #[allow(non_snake_case)]
                #[inline]
                pub fn set_${property.ident}(&mut self) {
                    self.set(${i})
                }
            % endif
        % endfor
    }
}

% for property in LONGHANDS:
    % if property.derived_from is None:
        #[allow(non_snake_case)]
        fn substitute_variables_${property.ident}<F>(
            value: &DeclaredValue<longhands::${property.ident}::SpecifiedValue>,
            custom_properties: &Option<Arc<::custom_properties::ComputedValuesMap>>,
            f: F,
            error_reporter: &mut Box<ParseErrorReporter + Send>)
            where F: FnOnce(&DeclaredValue<longhands::${property.ident}::SpecifiedValue>)
        {
            if let DeclaredValue::WithVariables {
                ref css, first_token_type, ref base_url, from_shorthand
            } = *value {
                substitute_variables_${property.ident}_slow(css,
                                                            first_token_type,
                                                            base_url,
                                                            from_shorthand,
                                                            custom_properties,
                                                            f,
                                                            error_reporter);
            } else {
                f(value);
            }
        }

        #[allow(non_snake_case)]
        #[inline(never)]
        fn substitute_variables_${property.ident}_slow<F>(
                css: &String,
                first_token_type: TokenSerializationType,
                base_url: &Url,
                from_shorthand: Option<Shorthand>,
                custom_properties: &Option<Arc<::custom_properties::ComputedValuesMap>>,
                f: F,
                error_reporter: &mut Box<ParseErrorReporter + Send>)
                where F: FnOnce(&DeclaredValue<longhands::${property.ident}::SpecifiedValue>) {
            f(&
                ::custom_properties::substitute(css, first_token_type, custom_properties)
                .and_then(|css| {
                    // As of this writing, only the base URL is used for property values:
                    //
                    // FIXME(pcwalton): Cloning the error reporter is slow! But so are custom
                    // properties, so whatever...
                    let context = ParserContext::new(
                        ::stylesheets::Origin::Author, base_url, (*error_reporter).clone());
                    Parser::new(&css).parse_entirely(|input| {
                        match from_shorthand {
                            None => {
                                longhands::${property.ident}::parse_specified(&context, input)
                            }
                            % for shorthand in SHORTHANDS:
                                % if property in shorthand.sub_properties:
                                    Some(Shorthand::${shorthand.camel_case}) => {
                                        shorthands::${shorthand.ident}::parse_value(&context, input)
                                        .map(|result| match result.${property.ident} {
                                            Some(value) => DeclaredValue::Value(value),
                                            None => DeclaredValue::Initial,
                                        })
                                    }
                                % endif
                            % endfor
                            _ => unreachable!()
                        }
                    })
                })
                .unwrap_or(
                    // Invalid at computed-value time.
                    DeclaredValue::${"Inherit" if property.style_struct.inherited else "Initial"}
                )
            );
        }
    % endif
% endfor

/// Declarations are stored in reverse order.
/// Overridden declarations are skipped.
#[derive(Debug, PartialEq, HeapSizeOf)]
pub struct PropertyDeclarationBlock {
    #[ignore_heap_size_of = "#7038"]
    pub important: Arc<Vec<PropertyDeclaration>>,
    #[ignore_heap_size_of = "#7038"]
    pub normal: Arc<Vec<PropertyDeclaration>>,
}

pub fn parse_style_attribute(input: &str, base_url: &Url, error_reporter: Box<ParseErrorReporter + Send>)
                             -> PropertyDeclarationBlock {
    let context = ParserContext::new(Origin::Author, base_url, error_reporter);
    parse_property_declaration_list(&context, &mut Parser::new(input))
}

pub fn parse_one_declaration(name: &str, input: &str, base_url: &Url, error_reporter: Box<ParseErrorReporter + Send>)
                             -> Result<Vec<PropertyDeclaration>, ()> {
    let context = ParserContext::new(Origin::Author, base_url, error_reporter);
    let mut results = vec![];
    match PropertyDeclaration::parse(name, &context, &mut Parser::new(input), &mut results) {
        PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => Ok(results),
        _ => Err(())
    }
}

struct PropertyDeclarationParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
}


/// Default methods reject all at rules.
impl<'a, 'b> AtRuleParser for PropertyDeclarationParser<'a, 'b> {
    type Prelude = ();
    type AtRule = (Vec<PropertyDeclaration>, bool);
}


impl<'a, 'b> DeclarationParser for PropertyDeclarationParser<'a, 'b> {
    type Declaration = (Vec<PropertyDeclaration>, bool);

    fn parse_value(&self, name: &str, input: &mut Parser) -> Result<(Vec<PropertyDeclaration>, bool), ()> {
        let mut results = vec![];
        try!(input.parse_until_before(Delimiter::Bang, |input| {
            match PropertyDeclaration::parse(name, self.context, input, &mut results) {
                PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => Ok(()),
                _ => Err(())
            }
        }));
        let important = input.try(parse_important).is_ok();
        Ok((results, important))
    }
}


pub fn parse_property_declaration_list(context: &ParserContext, input: &mut Parser)
                                       -> PropertyDeclarationBlock {
    let mut important_declarations = Vec::new();
    let mut normal_declarations = Vec::new();
    let parser = PropertyDeclarationParser {
        context: context,
    };
    let mut iter = DeclarationListParser::new(input, parser);
    while let Some(declaration) = iter.next() {
        match declaration {
            Ok((results, important)) => {
                if important {
                    important_declarations.extend(results);
                } else {
                    normal_declarations.extend(results);
                }
            }
            Err(range) => {
                let pos = range.start;
                let message = format!("Unsupported property declaration: '{}'",
                                      iter.input.slice(range));
                log_css_error(iter.input, pos, &*message, &context);
            }
        }
    }
    PropertyDeclarationBlock {
        important: Arc::new(deduplicate_property_declarations(important_declarations)),
        normal: Arc::new(deduplicate_property_declarations(normal_declarations)),
    }
}


/// Only keep the last declaration for any given property.
/// The input is in source order, output in reverse source order.
fn deduplicate_property_declarations(declarations: Vec<PropertyDeclaration>)
                                     -> Vec<PropertyDeclaration> {
    let mut deduplicated = vec![];
    let mut seen = PropertyBitField::new();
    let mut seen_custom = Vec::new();
    for declaration in declarations.into_iter().rev() {
        match declaration {
            % for property in LONGHANDS:
                PropertyDeclaration::${property.camel_case}(..) => {
                    % if property.derived_from is None:
                        if seen.get_${property.ident}() {
                            continue
                        }
                        seen.set_${property.ident}()
                    % else:
                        unreachable!();
                    % endif
                },
            % endfor
            PropertyDeclaration::Custom(ref name, _) => {
                if seen_custom.contains(name) {
                    continue
                }
                seen_custom.push(name.clone())
            }
        }
        deduplicated.push(declaration)
    }
    deduplicated
}


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CSSWideKeyword {
    InitialKeyword,
    InheritKeyword,
    UnsetKeyword,
}

impl CSSWideKeyword {
    pub fn parse(input: &mut Parser) -> Result<CSSWideKeyword, ()> {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            "initial" => Ok(CSSWideKeyword::InitialKeyword),
            "inherit" => Ok(CSSWideKeyword::InheritKeyword),
            "unset" => Ok(CSSWideKeyword::UnsetKeyword),
            _ => Err(())
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, HeapSizeOf)]
pub enum Shorthand {
    % for property in SHORTHANDS:
        ${property.camel_case},
    % endfor
}

impl Shorthand {
    pub fn from_name(name: &str) -> Option<Shorthand> {
        match_ignore_ascii_case! { name,
            % for property in SHORTHANDS:
                "${property.name}" => Some(Shorthand::${property.camel_case}),
            % endfor
            _ => None
        }
    }

    pub fn longhands(&self) -> &'static [&'static str] {
        % for property in SHORTHANDS:
            static ${property.ident.upper()}: &'static [&'static str] = &[
                % for sub in property.sub_properties:
                    "${sub.name}",
                % endfor
            ];
        % endfor
        match *self {
            % for property in SHORTHANDS:
                Shorthand::${property.camel_case} => ${property.ident.upper()},
            % endfor
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, HeapSizeOf)]
pub enum DeclaredValue<T> {
    Value(T),
    WithVariables {
        css: String,
        first_token_type: TokenSerializationType,
        base_url: Url,
        from_shorthand: Option<Shorthand>,
    },
    Initial,
    Inherit,
    // There is no Unset variant here.
    // The 'unset' keyword is represented as either Initial or Inherit,
    // depending on whether the property is inherited.
}

impl<T: ToCss> ToCss for DeclaredValue<T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            DeclaredValue::Value(ref inner) => inner.to_css(dest),
            DeclaredValue::WithVariables { ref css, from_shorthand: None, .. } => {
                dest.write_str(css)
            }
            // https://drafts.csswg.org/css-variables/#variables-in-shorthands
            DeclaredValue::WithVariables { .. } => Ok(()),
            DeclaredValue::Initial => dest.write_str("initial"),
            DeclaredValue::Inherit => dest.write_str("inherit"),
        }
    }
}

#[derive(PartialEq, Clone, Debug, HeapSizeOf)]
pub enum PropertyDeclaration {
    % for property in LONGHANDS:
        ${property.camel_case}(DeclaredValue<longhands::${property.ident}::SpecifiedValue>),
    % endfor
    Custom(::custom_properties::Name, DeclaredValue<::custom_properties::SpecifiedValue>),
}


#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PropertyDeclarationParseResult {
    UnknownProperty,
    ExperimentalProperty,
    InvalidValue,
    ValidOrIgnoredDeclaration,
}

#[derive(Eq, PartialEq, Clone)]
pub enum PropertyDeclarationName {
    Longhand(&'static str),
    Custom(::custom_properties::Name),
    Internal
}

impl PartialEq<str> for PropertyDeclarationName {
    fn eq(&self, other: &str) -> bool {
        match *self {
            PropertyDeclarationName::Longhand(n) => n == other,
            PropertyDeclarationName::Custom(ref n) => {
                ::custom_properties::parse_name(other) == Ok(&**n)
            }
            PropertyDeclarationName::Internal => false,
        }
    }
}

impl fmt::Display for PropertyDeclarationName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PropertyDeclarationName::Longhand(n) => f.write_str(n),
            PropertyDeclarationName::Custom(ref n) => {
                try!(f.write_str("--"));
                f.write_str(n)
            }
            PropertyDeclarationName::Internal => Ok(()),
        }
    }
}

impl PropertyDeclaration {
    pub fn name(&self) -> PropertyDeclarationName {
        match *self {
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    PropertyDeclaration::${property.camel_case}(..) => {
                        PropertyDeclarationName::Longhand("${property.name}")
                    }
                % endif
            % endfor
            PropertyDeclaration::Custom(ref name, _) => {
                PropertyDeclarationName::Custom(name.clone())
            }
            _ => PropertyDeclarationName::Internal,
        }
    }

    pub fn value(&self) -> String {
        match *self {
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    PropertyDeclaration::${property.camel_case}(ref value) =>
                        value.to_css_string(),
                % endif
            % endfor
            PropertyDeclaration::Custom(_, ref value) => value.to_css_string(),
            ref decl => panic!("unsupported property declaration: {}", decl.name()),
        }
    }

    /// If this is a pending-substitution value from the given shorthand, return that value
    // Extra space here because < seems to be removed by Mako when immediately followed by &.
    //                                                                          ↓
    pub fn with_variables_from_shorthand(&self, shorthand: Shorthand) -> Option< &str> {
        match *self {
            % for property in LONGHANDS:
                PropertyDeclaration::${property.camel_case}(ref value) => match *value {
                    DeclaredValue::WithVariables { ref css, from_shorthand: Some(s), .. }
                    if s == shorthand => {
                        Some(&**css)
                    }
                    _ => None
                },
            % endfor
            PropertyDeclaration::Custom(..) => None,
        }
    }

    /// Return whether this is a pending-substitution value.
    /// https://drafts.csswg.org/css-variables/#variables-in-shorthands
    pub fn with_variables(&self) -> bool {
        match *self {
            % for property in LONGHANDS:
                PropertyDeclaration::${property.camel_case}(ref value) => match *value {
                    DeclaredValue::WithVariables { .. } => true,
                    _ => false,
                },
            % endfor
            PropertyDeclaration::Custom(_, ref value) => match *value {
                DeclaredValue::WithVariables { .. } => true,
                _ => false,
            }
        }
    }

    pub fn matches(&self, name: &str) -> bool {
        match *self {
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    PropertyDeclaration::${property.camel_case}(..) => {
                        name.eq_ignore_ascii_case("${property.name}")
                    }
                % endif
            % endfor
            PropertyDeclaration::Custom(ref declaration_name, _) => {
                ::custom_properties::parse_name(name) == Ok(&**declaration_name)
            }
            _ => false,
        }
    }

    pub fn parse(name: &str, context: &ParserContext, input: &mut Parser,
                 result_list: &mut Vec<PropertyDeclaration>) -> PropertyDeclarationParseResult {
        if let Ok(name) = ::custom_properties::parse_name(name) {
            let value = match input.try(CSSWideKeyword::parse) {
                Ok(CSSWideKeyword::UnsetKeyword) |  // Custom properties are alawys inherited
                Ok(CSSWideKeyword::InheritKeyword) => DeclaredValue::Inherit,
                Ok(CSSWideKeyword::InitialKeyword) => DeclaredValue::Initial,
                Err(()) => match ::custom_properties::parse(input) {
                    Ok(value) => DeclaredValue::Value(value),
                    Err(()) => return PropertyDeclarationParseResult::InvalidValue,
                }
            };
            result_list.push(PropertyDeclaration::Custom(Atom::from(name), value));
            return PropertyDeclarationParseResult::ValidOrIgnoredDeclaration;
        }
        match_ignore_ascii_case! { name,
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    "${property.name}" => {
                        % if property.internal:
                            if context.stylesheet_origin != Origin::UserAgent {
                                return PropertyDeclarationParseResult::UnknownProperty
                            }
                        % endif
                        % if property.experimental:
                            if !::util::prefs::get_pref("${property.experimental}")
                                .as_boolean().unwrap_or(false) {
                                return PropertyDeclarationParseResult::ExperimentalProperty
                            }
                        % endif
                        match longhands::${property.ident}::parse_declared(context, input) {
                            Ok(value) => {
                                result_list.push(PropertyDeclaration::${property.camel_case}(value));
                                PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                            },
                            Err(()) => PropertyDeclarationParseResult::InvalidValue,
                        }
                    },
                % else:
                    "${property.name}" => PropertyDeclarationParseResult::UnknownProperty,
                % endif
            % endfor
            % for shorthand in SHORTHANDS:
                "${shorthand.name}" => {
                    % if shorthand.internal:
                        if context.stylesheet_origin != Origin::UserAgent {
                            return PropertyDeclarationParseResult::UnknownProperty
                        }
                    % endif
                    % if shorthand.experimental:
                        if !::util::prefs::get_pref("${shorthand.experimental}")
                            .as_boolean().unwrap_or(false) {
                            return PropertyDeclarationParseResult::ExperimentalProperty
                        }
                    % endif
                    match input.try(CSSWideKeyword::parse) {
                        Ok(CSSWideKeyword::InheritKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                result_list.push(
                                    PropertyDeclaration::${sub_property.camel_case}(
                                        DeclaredValue::Inherit));
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Ok(CSSWideKeyword::InitialKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                result_list.push(
                                    PropertyDeclaration::${sub_property.camel_case}(
                                        DeclaredValue::Initial));
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Ok(CSSWideKeyword::UnsetKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                result_list.push(PropertyDeclaration::${sub_property.camel_case}(
                                    DeclaredValue::${"Inherit" if sub_property.style_struct.inherited else "Initial"}
                                ));
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Err(()) => match shorthands::${shorthand.ident}::parse(context, input, result_list) {
                            Ok(()) => PropertyDeclarationParseResult::ValidOrIgnoredDeclaration,
                            Err(()) => PropertyDeclarationParseResult::InvalidValue,
                        }
                    }
                },
            % endfor

            _ => PropertyDeclarationParseResult::UnknownProperty
        }
    }
}

pub mod style_struct_traits {
    use super::longhands;

    % for style_struct in STYLE_STRUCTS:
        pub trait T${style_struct.trait_name}: Clone {
            % for longhand in style_struct.longhands:
                #[allow(non_snake_case)]
                fn set_${longhand.ident}(&mut self, v: longhands::${longhand.ident}::computed_value::T);
                #[allow(non_snake_case)]
                fn copy_${longhand.ident}_from(&mut self, other: &Self);
            % endfor
            % for additional in style_struct.additional_methods:
                #[allow(non_snake_case)]
                ${additional.declare()}
            % endfor
        }
    % endfor
}

pub mod style_structs {
    use fnv::FnvHasher;
    use super::longhands;
    use std::hash::{Hash, Hasher};

    % for style_struct in STYLE_STRUCTS:
        % if style_struct.trait_name == "Font":
        #[derive(Clone, HeapSizeOf, Debug)]
        % else:
        #[derive(PartialEq, Clone, HeapSizeOf)]
        % endif
        pub struct ${style_struct.servo_struct_name} {
            % for longhand in style_struct.longhands:
                pub ${longhand.ident}: longhands::${longhand.ident}::computed_value::T,
            % endfor
            % if style_struct.trait_name == "Font":
                pub hash: u64,
            % endif
        }
        % if style_struct.trait_name == "Font":

        impl PartialEq for ${style_struct.servo_struct_name} {
            fn eq(&self, other: &${style_struct.servo_struct_name}) -> bool {
                self.hash == other.hash
                % for longhand in style_struct.longhands:
                    && self.${longhand.ident} == other.${longhand.ident}
                % endfor
            }
        }
        % endif

        impl super::style_struct_traits::T${style_struct.trait_name} for ${style_struct.servo_struct_name} {
            % for longhand in style_struct.longhands:
                fn set_${longhand.ident}(&mut self, v: longhands::${longhand.ident}::computed_value::T) {
                    self.${longhand.ident} = v;
                }
                fn copy_${longhand.ident}_from(&mut self, other: &Self) {
                    self.${longhand.ident} = other.${longhand.ident}.clone();
                }
            % endfor
            % if style_struct.trait_name == "Border":
                % for side in ["top", "right", "bottom", "left"]:
                fn border_${side}_is_none_or_hidden_and_has_nonzero_width(&self) -> bool {
                    self.border_${side}_style.none_or_hidden() &&
                    self.border_${side}_width != ::app_units::Au(0)
                }
                % endfor
            % elif style_struct.trait_name == "Box":
                fn clone_display(&self) -> longhands::display::computed_value::T {
                    self.display.clone()
                }
                fn clone_position(&self) -> longhands::position::computed_value::T {
                    self.position.clone()
                }
                fn is_floated(&self) -> bool {
                    self.float != longhands::float::SpecifiedValue::none
                }
                fn overflow_x_is_visible(&self) -> bool {
                    self.overflow_x == longhands::overflow_x::computed_value::T::visible
                }
                fn overflow_y_is_visible(&self) -> bool {
                    self.overflow_y.0 == longhands::overflow_x::computed_value::T::visible
                }
                fn transition_count(&self) -> usize {
                    self.transition_property.0.len()
                }
            % elif style_struct.trait_name == "Color":
                fn clone_color(&self) -> longhands::color::computed_value::T {
                    self.color.clone()
                }
            % elif style_struct.trait_name == "Font":
                fn clone_font_size(&self) -> longhands::font_size::computed_value::T {
                    self.font_size.clone()
                }
                fn clone_font_weight(&self) -> longhands::font_weight::computed_value::T {
                    self.font_weight.clone()
                }
                fn compute_font_hash(&mut self) {
                    // Corresponds to the fields in `gfx::font_template::FontTemplateDescriptor`.
                    let mut hasher: FnvHasher = Default::default();
                    hasher.write_u16(self.font_weight as u16);
                    self.font_stretch.hash(&mut hasher);
                    self.font_family.hash(&mut hasher);
                    self.hash = hasher.finish()
                }
            % elif style_struct.trait_name == "InheritedBox":
                fn clone_direction(&self) -> longhands::direction::computed_value::T {
                    self.direction.clone()
                }
                fn clone_writing_mode(&self) -> longhands::writing_mode::computed_value::T {
                    self.writing_mode.clone()
                }
                fn clone_text_orientation(&self) -> longhands::text_orientation::computed_value::T {
                    self.text_orientation.clone()
                }
            % elif style_struct.trait_name == "InheritedText":
                fn clone__servo_text_decorations_in_effect(&self) ->
                    longhands::_servo_text_decorations_in_effect::computed_value::T {
                    self._servo_text_decorations_in_effect.clone()
                }
            % elif style_struct.trait_name == "Outline":
                fn outline_is_none_or_hidden_and_has_nonzero_width(&self) -> bool {
                    self.outline_style.none_or_hidden() && self.outline_width != ::app_units::Au(0)
                }
            % elif style_struct.trait_name == "Text":
                fn has_underline(&self) -> bool {
                    self.text_decoration.underline
                }
                fn has_overline(&self) -> bool {
                    self.text_decoration.overline
                }
                fn has_line_through(&self) -> bool {
                    self.text_decoration.line_through
                }
            % endif
        }

    % endfor
}

pub trait ComputedValues : Clone + Send + Sync + 'static {
    % for style_struct in STYLE_STRUCTS:
        type Concrete${style_struct.trait_name}: style_struct_traits::T${style_struct.trait_name};
    % endfor

        // Temporary bailout case for stuff we haven't made work with the trait
        // yet - panics for non-Servo implementations.
        //
        // Used only for animations. Don't use it in other places.
        fn as_servo<'a>(&'a self) -> &'a ServoComputedValues;
        fn as_servo_mut<'a>(&'a mut self) -> &'a mut ServoComputedValues;

        fn new(custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
               shareable: bool,
               writing_mode: WritingMode,
               root_font_size: Au,
        % for style_struct in STYLE_STRUCTS:
               ${style_struct.ident}: Arc<Self::Concrete${style_struct.trait_name}>,
        % endfor
        ) -> Self;

        fn initial_values() -> &'static Self;

        fn do_cascade_property<F: FnOnce(&Vec<Option<CascadePropertyFn<Self>>>)>(f: F);

    % for style_struct in STYLE_STRUCTS:
        fn clone_${style_struct.trait_name_lower}(&self) ->
            Arc<Self::Concrete${style_struct.trait_name}>;
        fn get_${style_struct.trait_name_lower}<'a>(&'a self) ->
            &'a Self::Concrete${style_struct.trait_name};
        fn mutate_${style_struct.trait_name_lower}<'a>(&'a mut self) ->
            &'a mut Self::Concrete${style_struct.trait_name};
    % endfor

    fn custom_properties(&self) -> Option<Arc<::custom_properties::ComputedValuesMap>>;
    fn root_font_size(&self) -> Au;
    fn set_root_font_size(&mut self, size: Au);
    fn set_writing_mode(&mut self, mode: WritingMode);
    fn is_multicol(&self) -> bool;
}

#[derive(Clone, HeapSizeOf)]
pub struct ServoComputedValues {
    % for style_struct in STYLE_STRUCTS:
        ${style_struct.ident}: Arc<style_structs::${style_struct.servo_struct_name}>,
    % endfor
    custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
    shareable: bool,
    pub writing_mode: WritingMode,
    pub root_font_size: Au,
}

impl ComputedValues for ServoComputedValues {
    % for style_struct in STYLE_STRUCTS:
        type Concrete${style_struct.trait_name} = style_structs::${style_struct.servo_struct_name};
    % endfor

        fn as_servo<'a>(&'a self) -> &'a ServoComputedValues { self }
        fn as_servo_mut<'a>(&'a mut self) -> &'a mut ServoComputedValues { self }

        fn new(custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
               shareable: bool,
               writing_mode: WritingMode,
               root_font_size: Au,
            % for style_struct in STYLE_STRUCTS:
               ${style_struct.ident}: Arc<style_structs::${style_struct.servo_struct_name}>,
            % endfor
        ) -> Self {
            ServoComputedValues {
                custom_properties: custom_properties,
                shareable: shareable,
                writing_mode: writing_mode,
                root_font_size: root_font_size,
            % for style_struct in STYLE_STRUCTS:
                ${style_struct.ident}: ${style_struct.ident},
            % endfor
            }
        }

        fn initial_values() -> &'static Self { &*INITIAL_SERVO_VALUES }

        fn do_cascade_property<F: FnOnce(&Vec<Option<CascadePropertyFn<Self>>>)>(f: F) {
            CASCADE_PROPERTY.with(|x| f(x));
        }

    % for style_struct in STYLE_STRUCTS:
        #[inline]
        fn clone_${style_struct.trait_name_lower}(&self) ->
            Arc<Self::Concrete${style_struct.trait_name}> {
                self.${style_struct.ident}.clone()
            }
        #[inline]
        fn get_${style_struct.trait_name_lower}<'a>(&'a self) ->
            &'a Self::Concrete${style_struct.trait_name} {
                &self.${style_struct.ident}
            }
        #[inline]
        fn mutate_${style_struct.trait_name_lower}<'a>(&'a mut self) ->
            &'a mut Self::Concrete${style_struct.trait_name} {
                Arc::make_mut(&mut self.${style_struct.ident})
            }
    % endfor

    // Cloning the Arc here is fine because it only happens in the case where we have custom
    // properties, and those are both rare and expensive.
    fn custom_properties(&self) -> Option<Arc<::custom_properties::ComputedValuesMap>> {
        self.custom_properties.as_ref().map(|x| x.clone())
    }

    fn root_font_size(&self) -> Au { self.root_font_size }
    fn set_root_font_size(&mut self, size: Au) { self.root_font_size = size }
    fn set_writing_mode(&mut self, mode: WritingMode) { self.writing_mode = mode; }

    #[inline]
    fn is_multicol(&self) -> bool {
        let style = self.get_column();
        style.column_count.0.is_some() || style.column_width.0.is_some()
    }
}

impl ServoComputedValues {
    /// Resolves the currentColor keyword.
    /// Any color value form computed values (except for the 'color' property itself)
    /// should go through this method.
    ///
    /// Usage example:
    /// let top_color = style.resolve_color(style.Border.border_top_color);
    #[inline]
    pub fn resolve_color(&self, color: Color) -> RGBA {
        match color {
            Color::RGBA(rgba) => rgba,
            Color::CurrentColor => self.get_color().color,
        }
    }

    #[inline]
    pub fn content_inline_size(&self) -> computed::LengthOrPercentageOrAuto {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() {
            box_style.height
        } else {
            box_style.width
        }
    }

    #[inline]
    pub fn content_block_size(&self) -> computed::LengthOrPercentageOrAuto {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.width } else { box_style.height }
    }

    #[inline]
    pub fn min_inline_size(&self) -> computed::LengthOrPercentage {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.min_height } else { position_style.min_width }
    }

    #[inline]
    pub fn min_block_size(&self) -> computed::LengthOrPercentage {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.min_width } else { position_style.min_height }
    }

    #[inline]
    pub fn max_inline_size(&self) -> computed::LengthOrPercentageOrNone {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.max_height } else { position_style.max_width }
    }

    #[inline]
    pub fn max_block_size(&self) -> computed::LengthOrPercentageOrNone {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.max_width } else { position_style.max_height }
    }

    #[inline]
    pub fn logical_padding(&self) -> LogicalMargin<computed::LengthOrPercentage> {
        let padding_style = self.get_padding();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            padding_style.padding_top,
            padding_style.padding_right,
            padding_style.padding_bottom,
            padding_style.padding_left,
        ))
    }

    #[inline]
    pub fn logical_border_width(&self) -> LogicalMargin<Au> {
        let border_style = self.get_border();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            border_style.border_top_width,
            border_style.border_right_width,
            border_style.border_bottom_width,
            border_style.border_left_width,
        ))
    }

    #[inline]
    pub fn logical_margin(&self) -> LogicalMargin<computed::LengthOrPercentageOrAuto> {
        let margin_style = self.get_margin();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            margin_style.margin_top,
            margin_style.margin_right,
            margin_style.margin_bottom,
            margin_style.margin_left,
        ))
    }

    #[inline]
    pub fn logical_position(&self) -> LogicalMargin<computed::LengthOrPercentageOrAuto> {
        // FIXME(SimonSapin): should be the writing mode of the containing block, maybe?
        let position_style = self.get_position();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            position_style.top,
            position_style.right,
            position_style.bottom,
            position_style.left,
        ))
    }

    #[inline]
    pub fn get_font_arc(&self) -> Arc<style_structs::ServoFont> {
        self.font.clone()
    }

    // http://dev.w3.org/csswg/css-transforms/#grouping-property-values
    pub fn get_used_transform_style(&self) -> computed_values::transform_style::T {
        use computed_values::mix_blend_mode;
        use computed_values::transform_style;

        let effects = self.get_effects();

        // TODO(gw): Add clip-path, isolation, mask-image, mask-border-source when supported.
        if effects.opacity < 1.0 ||
           !effects.filter.is_empty() ||
           effects.clip.0.is_some() {
           effects.mix_blend_mode != mix_blend_mode::T::normal ||
            return transform_style::T::flat;
        }

        if effects.transform_style == transform_style::T::auto {
            if effects.transform.0.is_some() {
                return transform_style::T::flat;
            }
            if effects.perspective != computed::LengthOrNone::None {
                return transform_style::T::flat;
            }
        }

        // Return the computed value if not overridden by the above exceptions
        effects.transform_style
    }

    pub fn transform_requires_layer(&self) -> bool {
        // Check if the transform matrix is 2D or 3D
        if let Some(ref transform_list) = self.get_effects().transform.0 {
            for transform in transform_list {
                match *transform {
                    computed_values::transform::ComputedOperation::Perspective(..) => {
                        return true;
                    }
                    computed_values::transform::ComputedOperation::Matrix(m) => {
                        // See http://dev.w3.org/csswg/css-transforms/#2d-matrix
                        if m.m31 != 0.0 || m.m32 != 0.0 ||
                           m.m13 != 0.0 || m.m23 != 0.0 ||
                           m.m43 != 0.0 || m.m14 != 0.0 ||
                           m.m24 != 0.0 || m.m34 != 0.0 ||
                           m.m33 != 1.0 || m.m44 != 1.0 {
                            return true;
                        }
                    }
                    computed_values::transform::ComputedOperation::Translate(_, _, z) => {
                        if z != Au(0) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Neither perspective nor transform present
        false
    }

    pub fn computed_value_to_string(&self, name: &str) -> Result<String, ()> {
        match name {
            % for style_struct in STYLE_STRUCTS:
                % for longhand in style_struct.longhands:
                "${longhand.name}" => Ok(self.${style_struct.ident}.${longhand.ident}.to_css_string()),
                % endfor
            % endfor
            _ => {
                let name = try!(::custom_properties::parse_name(name));
                let map = try!(self.custom_properties.as_ref().ok_or(()));
                let value = try!(map.get(&Atom::from(name)).ok_or(()));
                Ok(value.to_css_string())
            }
        }
    }
}


/// Return a WritingMode bitflags from the relevant CSS properties.
pub fn get_writing_mode<S: style_struct_traits::TInheritedBox>(inheritedbox_style: &S) -> WritingMode {
    use logical_geometry;
    let mut flags = WritingMode::empty();
    match inheritedbox_style.clone_direction() {
        computed_values::direction::T::ltr => {},
        computed_values::direction::T::rtl => {
            flags.insert(logical_geometry::FLAG_RTL);
        },
    }
    match inheritedbox_style.clone_writing_mode() {
        computed_values::writing_mode::T::horizontal_tb => {},
        computed_values::writing_mode::T::vertical_rl => {
            flags.insert(logical_geometry::FLAG_VERTICAL);
        },
        computed_values::writing_mode::T::vertical_lr => {
            flags.insert(logical_geometry::FLAG_VERTICAL);
            flags.insert(logical_geometry::FLAG_VERTICAL_LR);
        },
    }
    match inheritedbox_style.clone_text_orientation() {
        computed_values::text_orientation::T::sideways_right => {},
        computed_values::text_orientation::T::sideways_left => {
            flags.insert(logical_geometry::FLAG_VERTICAL_LR);
        },
        computed_values::text_orientation::T::sideways => {
            if flags.intersects(logical_geometry::FLAG_VERTICAL_LR) {
                flags.insert(logical_geometry::FLAG_SIDEWAYS_LEFT);
            }
        },
    }
    flags
}


/// The initial values for all style structs as defined by the specification.
lazy_static! {
    pub static ref INITIAL_SERVO_VALUES: ServoComputedValues = ServoComputedValues {
        % for style_struct in STYLE_STRUCTS:
            ${style_struct.ident}: Arc::new(style_structs::${style_struct.servo_struct_name} {
                % for longhand in style_struct.longhands:
                    ${longhand.ident}: longhands::${longhand.ident}::get_initial_value(),
                % endfor
                % if style_struct.trait_name == "Font":
                    hash: 0,
                % endif
            }),
        % endfor
        custom_properties: None,
        shareable: true,
        writing_mode: WritingMode::empty(),
        root_font_size: longhands::font_size::get_initial_value(),
    };
}


/// Fast path for the function below. Only computes new inherited styles.
#[allow(unused_mut, unused_imports)]
fn cascade_with_cached_declarations<C: ComputedValues>(
        viewport_size: Size2D<Au>,
        applicable_declarations: &[DeclarationBlock<Vec<PropertyDeclaration>>],
        shareable: bool,
        parent_style: &C,
        cached_style: &C,
        custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
        mut error_reporter: Box<ParseErrorReporter + Send>)
        -> C {
    let mut context = computed::Context {
        is_root_element: false,
        viewport_size: viewport_size,
        inherited_style: parent_style,
        style: C::new(
            custom_properties,
            shareable,
            WritingMode::empty(),
            parent_style.root_font_size(),
            % for style_struct in STYLE_STRUCTS:
                % if style_struct.inherited:
                    parent_style
                % else:
                    cached_style
                % endif
                    .clone_${style_struct.trait_name_lower}(),
            % endfor
        ),
    };
    let mut seen = PropertyBitField::new();
    // Declaration blocks are stored in increasing precedence order,
    // we want them in decreasing order here.
    for sub_list in applicable_declarations.iter().rev() {
        // Declarations are already stored in reverse order.
        for declaration in sub_list.declarations.iter() {
            match *declaration {
                % for style_struct in STYLE_STRUCTS:
                    % for property in style_struct.longhands:
                        % if property.derived_from is None:
                            PropertyDeclaration::${property.camel_case}(ref
                                    ${'_' if not style_struct.inherited else ''}declared_value)
                                    => {
                                    use properties::style_struct_traits::T${style_struct.trait_name};
                                % if style_struct.inherited:
                                    if seen.get_${property.ident}() {
                                        continue
                                    }
                                    seen.set_${property.ident}();
                                    let custom_props = context.style().custom_properties();
                                    substitute_variables_${property.ident}(
                                        declared_value, &custom_props,
                                        |value| match *value {
                                            DeclaredValue::Value(ref specified_value)
                                            => {
                                                let computed = specified_value.to_computed_value(&context);
                                                context.mutate_style().mutate_${style_struct.trait_name_lower}()
                                                       .set_${property.ident}(computed);
                                            },
                                            DeclaredValue::Initial
                                            => {
                                                // FIXME(bholley): We may want set_X_to_initial_value() here.
                                                let initial = longhands::${property.ident}::get_initial_value();
                                                context.mutate_style().mutate_${style_struct.trait_name_lower}()
                                                       .set_${property.ident}(initial);
                                            },
                                            DeclaredValue::Inherit => {
                                                // This is a bit slow, but this is rare so it shouldn't
                                                // matter.
                                                //
                                                // FIXME: is it still?
                                                let inherited_struct = parent_style.get_${style_struct.ident}();
                                                context.mutate_style().mutate_${style_struct.trait_name_lower}()
                                                       .copy_${property.ident}_from(inherited_struct);
                                            }
                                            DeclaredValue::WithVariables { .. } => unreachable!()
                                        }, &mut error_reporter
                                    );
                                % endif

                                % if property.name in DERIVED_LONGHANDS:
                                    % for derived in DERIVED_LONGHANDS[property.name]:
                                            longhands::${derived.ident}
                                                     ::derive_from_${property.ident}(&mut context);
                                    % endfor
                                % endif
                            }
                        % else:
                            PropertyDeclaration::${property.camel_case}(_) => {
                                // Do not allow stylesheets to set derived properties.
                            }
                        % endif
                    % endfor
                % endfor
                PropertyDeclaration::Custom(..) => {}
            }
        }
    }

    if seen.get_font_style() || seen.get_font_weight() || seen.get_font_stretch() ||
            seen.get_font_family() {
        use properties::style_struct_traits::TFont;
        context.mutate_style().mutate_font().compute_font_hash();
    }

    context.style
}

pub type CascadePropertyFn<C /*: ComputedValues */> =
    extern "Rust" fn(declaration: &PropertyDeclaration,
                     inherited_style: &C,
                     context: &mut computed::Context<C>,
                     seen: &mut PropertyBitField,
                     cacheable: &mut bool,
                     error_reporter: &mut Box<ParseErrorReporter + Send>);

pub fn make_cascade_vec<C: ComputedValues>() -> Vec<Option<CascadePropertyFn<C>>> {
    let mut result: Vec<Option<CascadePropertyFn<C>>> = Vec::new();
    % for style_struct in STYLE_STRUCTS:
        % for property in style_struct.longhands:
            let discriminant;
            unsafe {
                let variant = PropertyDeclaration::${property.camel_case}(intrinsics::uninit());
                discriminant = intrinsics::discriminant_value(&variant) as usize;
                mem::forget(variant);
            }
            while result.len() < discriminant + 1 {
                result.push(None)
            }
            result[discriminant] = Some(longhands::${property.ident}::cascade_property);
        % endfor
    % endfor
    result
}

// This is a thread-local rather than a lazy static to avoid atomic operations when cascading
// properties.
thread_local!(static CASCADE_PROPERTY: Vec<Option<CascadePropertyFn<ServoComputedValues>>> = {
    make_cascade_vec::<ServoComputedValues>()
});

/// Performs the CSS cascade, computing new styles for an element from its parent style and
/// optionally a cached related style. The arguments are:
///
///   * `viewport_size`: The size of the initial viewport.
///
///   * `applicable_declarations`: The list of CSS rules that matched.
///
///   * `shareable`: Whether the `ComputedValues` structure to be constructed should be considered
///     shareable.
///
///   * `parent_style`: The parent style, if applicable; if `None`, this is the root node.
///
///   * `cached_style`: If present, cascading is short-circuited for everything but inherited
///     values and these values are used instead. Obviously, you must be careful when supplying
///     this that it is safe to only provide inherited declarations. If `parent_style` is `None`,
///     this is ignored.
///
/// Returns the computed values and a boolean indicating whether the result is cacheable.
pub fn cascade<C: ComputedValues>(
               viewport_size: Size2D<Au>,
               applicable_declarations: &[DeclarationBlock<Vec<PropertyDeclaration>>],
               shareable: bool,
               parent_style: Option<<&C>,
               cached_style: Option<<&C>,
               mut error_reporter: Box<ParseErrorReporter + Send>)
               -> (C, bool) {
    use properties::style_struct_traits::{TBorder, TBox, TColor, TFont, TOutline};
    let initial_values = C::initial_values();
    let (is_root_element, inherited_style) = match parent_style {
        Some(parent_style) => (false, parent_style),
        None => (true, initial_values),
    };

    let inherited_custom_properties = inherited_style.custom_properties();
    let mut custom_properties = None;
    let mut seen_custom = HashSet::new();
    for sub_list in applicable_declarations.iter().rev() {
        // Declarations are already stored in reverse order.
        for declaration in sub_list.declarations.iter() {
            match *declaration {
                PropertyDeclaration::Custom(ref name, ref value) => {
                    ::custom_properties::cascade(
                        &mut custom_properties, &inherited_custom_properties,
                        &mut seen_custom, name, value)
                }
                _ => {}
            }
        }
    }
    let custom_properties = ::custom_properties::finish_cascade(
            custom_properties, &inherited_custom_properties);

    if let (Some(cached_style), Some(parent_style)) = (cached_style, parent_style) {
        let style = cascade_with_cached_declarations(viewport_size,
                                                     applicable_declarations,
                                                     shareable,
                                                     parent_style,
                                                     cached_style,
                                                     custom_properties,
                                                     error_reporter);
        return (style, false)
    }

    let mut context = computed::Context {
        is_root_element: is_root_element,
        viewport_size: viewport_size,
        inherited_style: inherited_style,
        style: C::new(
            custom_properties,
            shareable,
            WritingMode::empty(),
            inherited_style.root_font_size(),
            % for style_struct in STYLE_STRUCTS:
            % if style_struct.inherited:
            inherited_style
            % else:
            initial_values
            % endif
                .clone_${style_struct.trait_name_lower}(),
            % endfor
        ),
    };

    // Set computed values, overwriting earlier declarations for the same property.
    let mut cacheable = true;
    let mut seen = PropertyBitField::new();
    // Declaration blocks are stored in increasing precedence order, we want them in decreasing
    // order here.
    //
    // We could (and used to) use a pattern match here, but that bloats this function to over 100K
    // of compiled code! To improve i-cache behavior, we outline the individual functions and use
    // virtual dispatch instead.
    C::do_cascade_property(|cascade_property| {
        % for category_to_cascade_now in ["early", "other"]:
            for sub_list in applicable_declarations.iter().rev() {
                // Declarations are already stored in reverse order.
                for declaration in sub_list.declarations.iter() {
                    if let PropertyDeclaration::Custom(..) = *declaration {
                        continue
                    }
                    // The computed value of some properties depends on the (sometimes computed)
                    // value of *other* properties.
                    // So we classify properties into "early" and "other",
                    // such that the only dependencies can be from "other" to "early".
                    // We iterate applicable_declarations twice, first cascading "early" properties
                    // then "other".
                    // Unfortunately, it’s not easy to check that this classification is correct.
                    let is_early_property = matches!(*declaration,
                        PropertyDeclaration::FontSize(_) |
                        PropertyDeclaration::Color(_) |
                        PropertyDeclaration::Position(_) |
                        PropertyDeclaration::Float(_) |
                        PropertyDeclaration::TextDecoration(_)
                    );
                    if
                        % if category_to_cascade_now == "early":
                            !
                        % endif
                        is_early_property
                    {
                        continue
                    }
                    let discriminant = unsafe {
                        intrinsics::discriminant_value(declaration) as usize
                    };
                    (cascade_property[discriminant].unwrap())(declaration,
                                                              inherited_style,
                                                              &mut context,
                                                              &mut seen,
                                                              &mut cacheable,
                                                              &mut error_reporter);
                }
            }
        % endfor
    });

    let mut style = context.style;

    let positioned = matches!(style.get_box().clone_position(),
        longhands::position::SpecifiedValue::absolute |
        longhands::position::SpecifiedValue::fixed);
    let floated = style.get_box().is_floated();
    if positioned || floated || is_root_element {
        use computed_values::display::T;

        let specified_display = style.get_box().clone_display();
        let computed_display = match specified_display {
            T::inline_table => {
                Some(T::table)
            }
            T::inline | T::inline_block |
            T::table_row_group | T::table_column |
            T::table_column_group | T::table_header_group |
            T::table_footer_group | T::table_row | T::table_cell |
            T::table_caption => {
                Some(T::block)
            }
            _ => None
        };
        if let Some(computed_display) = computed_display {
            let box_ = style.mutate_box();
            box_.set_display(computed_display);
            box_.set__servo_display_for_hypothetical_box(if is_root_element {
                computed_display
            } else {
                specified_display
            });
        }
    }

    {
        use computed_values::overflow_x::T as overflow;
        use computed_values::overflow_y;
        match (style.get_box().overflow_x_is_visible(), style.get_box().overflow_y_is_visible()) {
            (true, true) => {}
            (true, _) => {
                style.mutate_box().set_overflow_x(overflow::auto);
            }
            (_, true) => {
                style.mutate_box().set_overflow_y(overflow_y::T(overflow::auto));
            }
            _ => {}
        }
    }

    // The initial value of border-*-width may be changed at computed value time.
    % for side in ["top", "right", "bottom", "left"]:
        // Like calling to_computed_value, which wouldn't type check.
        if style.get_border().border_${side}_is_none_or_hidden_and_has_nonzero_width() {
            style.mutate_border().set_border_${side}_width(Au(0));
        }
    % endfor

    // The initial value of outline width may be changed at computed value time.
    if style.get_outline().outline_is_none_or_hidden_and_has_nonzero_width() {
        style.mutate_outline().set_outline_width(Au(0));
    }

    if is_root_element {
        let s = style.get_font().clone_font_size();
        style.set_root_font_size(s);
    }

    if seen.get_font_style() || seen.get_font_weight() || seen.get_font_stretch() ||
            seen.get_font_family() {
        use properties::style_struct_traits::TFont;
        style.mutate_font().compute_font_hash();
    }

    let mode = get_writing_mode(style.get_inheritedbox());
    style.set_writing_mode(mode);
    (style, cacheable)
}

/// Alters the given style to accommodate replaced content. This is called in flow construction. It
/// handles cases like `<div style="position: absolute">foo bar baz</div>` (in which `foo`, `bar`,
/// and `baz` must not be absolutely-positioned) and cases like `<sup>Foo</sup>` (in which the
/// `vertical-align: top` style of `sup` must not propagate down into `Foo`).
///
/// FIXME(#5625, pcwalton): It would probably be cleaner and faster to do this in the cascade.
#[inline]
pub fn modify_style_for_replaced_content(style: &mut Arc<ServoComputedValues>) {
    // Reset `position` to handle cases like `<div style="position: absolute">foo bar baz</div>`.
    if style.box_.display != longhands::display::computed_value::T::inline {
        let mut style = Arc::make_mut(style);
        Arc::make_mut(&mut style.box_).display = longhands::display::computed_value::T::inline;
        Arc::make_mut(&mut style.box_).position =
            longhands::position::computed_value::T::static_;
    }

    // Reset `vertical-align` to handle cases like `<sup>foo</sup>`.
    if style.box_.vertical_align != longhands::vertical_align::computed_value::T::baseline {
        let mut style = Arc::make_mut(style);
        Arc::make_mut(&mut style.box_).vertical_align =
            longhands::vertical_align::computed_value::T::baseline
    }

    // Reset margins.
    if style.margin.margin_top != computed::LengthOrPercentageOrAuto::Length(Au(0)) ||
            style.margin.margin_left != computed::LengthOrPercentageOrAuto::Length(Au(0)) ||
            style.margin.margin_bottom != computed::LengthOrPercentageOrAuto::Length(Au(0)) ||
            style.margin.margin_right != computed::LengthOrPercentageOrAuto::Length(Au(0)) {
        let mut style = Arc::make_mut(style);
        let margin = Arc::make_mut(&mut style.margin);
        margin.margin_top = computed::LengthOrPercentageOrAuto::Length(Au(0));
        margin.margin_left = computed::LengthOrPercentageOrAuto::Length(Au(0));
        margin.margin_bottom = computed::LengthOrPercentageOrAuto::Length(Au(0));
        margin.margin_right = computed::LengthOrPercentageOrAuto::Length(Au(0));
    }
}

/// Adjusts borders as appropriate to account for a fragment's status as the first or last fragment
/// within the range of an element.
///
/// Specifically, this function sets border widths to zero on the sides for which the fragment is
/// not outermost.
#[inline]
pub fn modify_border_style_for_inline_sides(style: &mut Arc<ServoComputedValues>,
                                            is_first_fragment_of_element: bool,
                                            is_last_fragment_of_element: bool) {
    fn modify_side(style: &mut Arc<ServoComputedValues>, side: PhysicalSide) {
        {
            let border = &style.border;
            let current_style = match side {
                PhysicalSide::Left =>   (border.border_left_width,   border.border_left_style),
                PhysicalSide::Right =>  (border.border_right_width,  border.border_right_style),
                PhysicalSide::Top =>    (border.border_top_width,    border.border_top_style),
                PhysicalSide::Bottom => (border.border_bottom_width, border.border_bottom_style),
            };
            if current_style == (Au(0), BorderStyle::none) {
                return;
            }
        }
        let mut style = Arc::make_mut(style);
        let border = Arc::make_mut(&mut style.border);
        match side {
            PhysicalSide::Left => {
                border.border_left_width = Au(0);
                border.border_left_style = BorderStyle::none;
            }
            PhysicalSide::Right => {
                border.border_right_width = Au(0);
                border.border_right_style = BorderStyle::none;
            }
            PhysicalSide::Bottom => {
                border.border_bottom_width = Au(0);
                border.border_bottom_style = BorderStyle::none;
            }
            PhysicalSide::Top => {
                border.border_top_width = Au(0);
                border.border_top_style = BorderStyle::none;
            }
        }
    }

    if !is_first_fragment_of_element {
        let side = style.writing_mode.inline_start_physical_side();
        modify_side(style, side)
    }

    if !is_last_fragment_of_element {
        let side = style.writing_mode.inline_end_physical_side();
        modify_side(style, side)
    }
}

/// Adjusts the display and position properties as appropriate for an anonymous table object.
#[inline]
pub fn modify_style_for_anonymous_table_object(
        style: &mut Arc<ServoComputedValues>,
        new_display_value: longhands::display::computed_value::T) {
    let mut style = Arc::make_mut(style);
    let box_style = Arc::make_mut(&mut style.box_);
    box_style.display = new_display_value;
    box_style.position = longhands::position::computed_value::T::static_;
}

/// Adjusts the `position` property as necessary for the outer fragment wrapper of an inline-block.
#[inline]
pub fn modify_style_for_outer_inline_block_fragment(style: &mut Arc<ServoComputedValues>) {
    let mut style = Arc::make_mut(style);
    let box_style = Arc::make_mut(&mut style.box_);
    box_style.position = longhands::position::computed_value::T::static_
}

/// Adjusts the `position` and `padding` properties as necessary to account for text.
///
/// Text is never directly relatively positioned; it's always contained within an element that is
/// itself relatively positioned.
#[inline]
pub fn modify_style_for_text(style: &mut Arc<ServoComputedValues>) {
    if style.box_.position == longhands::position::computed_value::T::relative {
        // We leave the `position` property set to `relative` so that we'll still establish a
        // containing block if needed. But we reset all position offsets to `auto`.
        let mut style = Arc::make_mut(style);
        let mut position = Arc::make_mut(&mut style.position);
        position.top = computed::LengthOrPercentageOrAuto::Auto;
        position.right = computed::LengthOrPercentageOrAuto::Auto;
        position.bottom = computed::LengthOrPercentageOrAuto::Auto;
        position.left = computed::LengthOrPercentageOrAuto::Auto;
    }

    if style.padding.padding_top != computed::LengthOrPercentage::Length(Au(0)) ||
            style.padding.padding_right != computed::LengthOrPercentage::Length(Au(0)) ||
            style.padding.padding_bottom != computed::LengthOrPercentage::Length(Au(0)) ||
            style.padding.padding_left != computed::LengthOrPercentage::Length(Au(0)) {
        let mut style = Arc::make_mut(style);
        let mut padding = Arc::make_mut(&mut style.padding);
        padding.padding_top = computed::LengthOrPercentage::Length(Au(0));
        padding.padding_right = computed::LengthOrPercentage::Length(Au(0));
        padding.padding_bottom = computed::LengthOrPercentage::Length(Au(0));
        padding.padding_left = computed::LengthOrPercentage::Length(Au(0));
    }

    if style.effects.opacity != 1.0 {
        let mut style = Arc::make_mut(style);
        let mut effects = Arc::make_mut(&mut style.effects);
        effects.opacity = 1.0;
    }
}

/// Adjusts the `margin` property as necessary to account for the text of an `input` element.
///
/// Margins apply to the `input` element itself, so including them in the text will cause them to
/// be double-counted.
pub fn modify_style_for_input_text(style: &mut Arc<ServoComputedValues>) {
    let mut style = Arc::make_mut(style);
    let margin_style = Arc::make_mut(&mut style.margin);
    margin_style.margin_top = computed::LengthOrPercentageOrAuto::Length(Au(0));
    margin_style.margin_right = computed::LengthOrPercentageOrAuto::Length(Au(0));
    margin_style.margin_bottom = computed::LengthOrPercentageOrAuto::Length(Au(0));
    margin_style.margin_left = computed::LengthOrPercentageOrAuto::Length(Au(0));

    // whitespace inside text input should not be collapsed
    let inherited_text = Arc::make_mut(&mut style.inheritedtext);
    inherited_text.white_space = longhands::white_space::computed_value::T::pre;
}

/// Adjusts the `clip` property so that an inline absolute hypothetical fragment doesn't clip its
/// children.
pub fn modify_style_for_inline_absolute_hypothetical_fragment(style: &mut Arc<ServoComputedValues>) {
    if style.get_effects().clip.0.is_some() {
        let mut style = Arc::make_mut(style);
        let effects_style = Arc::make_mut(&mut style.effects);
        effects_style.clip.0 = None
    }
}

pub fn is_supported_property(property: &str) -> bool {
    match_ignore_ascii_case! { property,
        % for property in SHORTHANDS + LONGHANDS:
            "${property.name}" => true,
        % endfor
        _ => property.starts_with("--")
    }
}

#[macro_export]
macro_rules! css_properties_accessors {
    ($macro_name: ident) => {
        $macro_name! {
            % for property in SHORTHANDS + LONGHANDS:
                % if property.derived_from is None and not property.internal:
                    % if '-' in property.name:
                        [${property.ident.capitalize()}, Set${property.ident.capitalize()}, "${property.name}"],
                    % endif
                    % if property != LONGHANDS[-1]:
                        [${property.camel_case}, Set${property.camel_case}, "${property.name}"],
                    % else:
                        [${property.camel_case}, Set${property.camel_case}, "${property.name}"]
                    % endif
                % endif
            % endfor
        }
    }
}


macro_rules! longhand_properties_idents {
    ($macro_name: ident) => {
        $macro_name! {
            % for property in LONGHANDS:
                ${property.ident}
            % endfor
        }
    }
}
