<%page args="helpers"/>

<%helpers:longhand name="font-weight">
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
</%helpers:longhand>
