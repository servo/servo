<%page args="helpers"/>

<%helpers:longhand name="outline-style">
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
</%helpers:longhand>
