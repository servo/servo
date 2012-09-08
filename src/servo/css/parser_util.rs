#[doc = "Helper functions to parse values of specific attributes."]

use css::values::*;
use str::{pop_char, from_chars};
use float::from_str;
use option::map;

export parse_font_size;
export parse_size;
export parse_box_sizing;
export parse_display_type;


fn parse_length(str : ~str) -> Option<Length> {
    // TODO: use these once we stop lexing below
    const PTS_PER_INCH: float = 72.0;
    const CM_PER_INCH: float = 2.54;
    const PX_PER_PT: float = 1.0 / 0.75;

    match str {
      s if s.ends_with(~"in") => from_str(str.substr(0, str.len() - 2)).map(|f| Px(1.0/0.75 * 72.0 * f)),
      s if s.ends_with(~"cm") => from_str(str.substr(0, str.len() - 2)).map(|f| Px(f / 2.54 * 72.0 * 1.0/0.75)),
      s if s.ends_with(~"mm") => from_str(str.substr(0, str.len() - 2)).map(|f| Px(f * 0.1 / 2.54 * 72.0 * 1.0/0.75)),
      s if s.ends_with(~"pt") => from_str(str.substr(0, str.len() - 2)).map(|f| Px(1.0/0.75 * f)),
      s if s.ends_with(~"pc") => from_str(str.substr(0, str.len() - 2)).map(|f| Px(1.0/0.75 * 12.0 * f)),
      s if s.ends_with(~"px") => from_str(str.substr(0, str.len() - 2)).map(|f| Px(f)),
      s if s.ends_with(~"em") => from_str(str.substr(0, str.len() - 2)).map(|f| Em(f)),
      s if s.ends_with(~"ex") => from_str(str.substr(0, str.len() - 2)).map(|f| Em(0.5*f)),
      _ => None,
    }
}

fn parse_absolute_size(str : ~str) -> ParseResult<AbsoluteSize> {
    match str {
      ~"xx-small" => Value(XXSmall),
      ~"x-small" => Value(XSmall),
      ~"small" => Value(Small),
      ~"medium" => Value(Medium),
      ~"large" => Value(Large),
      ~"x-large" => Value(XLarge),
      ~"xx-large" => Value(XXLarge),
      _  => Fail
    }
}

fn parse_relative_size(str: ~str) -> ParseResult<RelativeSize> {
    match str {
      ~"smaller" => Value(Smaller),
      ~"larger" => Value(Larger),
      _ => Fail
    }
}

fn parse_font_size(str: ~str) -> ParseResult<CSSFontSize> {
    // TODO: complete me
    Value(LengthSize(Px(14.0)))
}

// For width / height, and anything else with the same attribute values
fn parse_box_sizing(str : ~str) -> ParseResult<BoxSizing> {
    match str {
      ~"auto" => Value(BoxAuto),
      ~"inherit" => CSSInherit,
      _ => Fail,
    }
}

fn parse_display_type(str : ~str) -> ParseResult<CSSDisplay> {
    match str {
      ~"inline" => Value(DisplayInline),
      ~"block" => Value(DisplayBlock),
      ~"none" => Value(DisplayNone),
      _ => { #debug["Recieved unknown display value '%s'", str]; Fail }
    }
}

#[cfg(test)]
mod test {
    import css::lexer::spawn_css_lexer_from_string;
    import css::parser::build_stylesheet;
    import css::values::{Stylesheet, Element, FontSize, Width, Height};
    
    #[test]
    fn should_match_font_sizes() {
        let input = ~"* {font-size:12pt; font-size:inherit; font-size:200%; font-size:x-small}";
        let token_port = spawn_css_lexer_from_string(input);
        let actual_rule = build_stylesheet(token_port);
        let expected_rule : Stylesheet = ~[~(~[~Element(~"*", ~[])],
                                             ~[FontSize(Pt(12.0)),
                                               FontSize(Percent(100.0)),
                                               FontSize(Percent(200.0)),
                                               FontSize(Px(12.0))])];

        assert actual_rule == expected_rule;
    }

    #[test]
    fn should_match_width_height() {
        let input = ~"* {width:20%; height:auto; width:20px; width:3in; height:70mm; height:3cm}";
        let token_port = spawn_css_lexer_from_string(input);
        let actual_rule = build_stylesheet(token_port);
        let expected_rule : Stylesheet = ~[~(~[~Element(~"*", ~[])],
                                             ~[Width(Percent(20.0)),
                                               Height(Auto),
                                               Width(Px(20.0)),
                                               Width(Pt(216.0)),
                                               Height(Mm(70.0)),
                                               Height(Mm(30.0))])];

        assert actual_rule == expected_rule;
    }
}
