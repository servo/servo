use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::values::specified::length::Length;
use style::parser::Parse;
use style::stylesheets::Origin;
use style_traits::ToCss;

#[test]
fn test_calc() {
    assert!(parse(Length::parse, "calc(1px+ 2px)").is_err());
}
