<%page args="helpers"/>

<%helpers:longhand name="text-align">
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
</%helpers:longhand>
