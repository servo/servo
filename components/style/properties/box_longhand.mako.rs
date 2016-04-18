<%page args="helpers" />

${helpers.new_style_struct("Box", is_inherited=False, gecko_name="nsStyleDisplay",
                           additional_methods=[
                                helpers.new_method("clone_display",
                                                   "longhands::display::computed_value::T"),
                                helpers.new_method("clone_position",
                                                   "longhands::position::computed_value::T"),
                                helpers.new_method("is_floated", "bool"),
                                helpers.new_method("overflow_x_is_visible", "bool"),
                                helpers.new_method("overflow_y_is_visible", "bool"),
                                helpers.new_method("transition_count", "usize")])}

// TODO(SimonSapin): don't parse `inline-table`, since we don't support it
<%helpers:longhand name="display" custom_cascade="${PRODUCT == 'servo'}">
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

    pub mod computed_value {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, Copy, Hash, RustcEncodable, Debug, HeapSizeOf)]
        #[derive(Deserialize, Serialize)]
        pub enum T {
            % for value in values:
                ${helpers.to_rust_ident(value)},
            % endfor
        }

        impl ::cssparser::ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
            where W: ::std::fmt::Write {
                match *self {
                    % for value in values:
                        T::${helpers.to_rust_ident(value)} => dest.write_str("${value}"),
                    % endfor
                }
            }
        }
    }
    #[inline] pub fn get_initial_value() -> computed_value::T {
        computed_value::T::${helpers.to_rust_ident(values[0])}
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
                    Ok(computed_value::T::${helpers.to_rust_ident(value)})
                },
            % endfor
            _ => Err(())
        }
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}

    % if PRODUCT == "servo":
        fn cascade_property_custom<C: ComputedValues>(
                                   _declaration: &PropertyDeclaration,
                                   _inherited_style: &C,
                                   context: &mut computed::Context<C>,
                                   _seen: &mut PropertyBitField,
                                   _cacheable: &mut bool,
                                   _error_reporter: &mut StdBox<ParseErrorReporter + Send>) {
            longhands::_servo_display_for_hypothetical_box::derive_from_display(context);
            longhands::_servo_text_decorations_in_effect::derive_from_display(context);
        }
    % endif

</%helpers:longhand>

${helpers.single_keyword("position", "static absolute relative fixed", extra_gecko_values="sticky")}

<%helpers:single_keyword_computed name="float" values="none left right" gecko_ffi_name="mFloats">
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

</%helpers:single_keyword_computed>

${helpers.single_keyword("clear", "none left right both", gecko_ffi_name="mBreakType")}

<%helpers:longhand name="-servo-display-for-hypothetical-box" derived_from="display" products="servo">
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

</%helpers:longhand>
