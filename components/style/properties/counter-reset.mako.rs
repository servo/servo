<%page args="helpers"/>

<%helpers:longhand name="counter-reset">
    pub use super::counter_increment::{SpecifiedValue, computed_value, get_initial_value};
    use super::counter_increment::{parse_common};

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        parse_common(0, input)
    }
</%helpers:longhand>
