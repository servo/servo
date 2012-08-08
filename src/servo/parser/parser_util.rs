#[doc = "Helper functions to parse values of specific attributes."]

import dom::style::*;
import str::{pop_char, from_chars};
import float::from_str;
import option::map;

export parse_font_size;
export parse_size;
export parse_display_type;

fn parse_unit(str : ~str) -> option<Unit> {
    match str {
      s if s.ends_with(~"%") => from_str(str.substr(0, str.len() - 1)).map(|f| Percent(f)),
      s if s.ends_with(~"in") => from_str(str.substr(0, str.len() - 2)).map(|f| In(f)),
      s if s.ends_with(~"cm") => from_str(str.substr(0, str.len() - 2)).map(|f| Cm(f)),
      s if s.ends_with(~"mm") => from_str(str.substr(0, str.len() - 2)).map(|f| Mm(f)),
      s if s.ends_with(~"pt") => from_str(str.substr(0, str.len() - 2)).map(|f| Pt(f)),
      s if s.ends_with(~"pc") => from_str(str.substr(0, str.len() - 2)).map(|f| Pc(f)),
      s if s.ends_with(~"px") => from_str(str.substr(0, str.len() - 2)).map(|f| Px(f)),
      s if s.ends_with(~"em") => from_str(str.substr(0, str.len() - 2)).map(|f| Em(f)),
      s if s.ends_with(~"ex") => from_str(str.substr(0, str.len() - 2)).map(|f| Ex(f)),
      _ => none,
    }
}

fn parse_font_size(str : ~str) -> option<Unit> {
    // The default pixel size, not sure if this is accurate.
    let default = 16.0;

    match str {
      ~"xx-small" => some(Px(0.6*default)),
      ~"x-small" => some(Px(0.75*default)),
      ~"small" => some(Px(8.0/9.0*default)),
      ~"medium" => some(Px(default)),
      ~"large" => some(Px(1.2*default)),
      ~"x-large" => some(Px(1.5*default)),
      ~"xx-large" => some(Px(2.0*default)),
      ~"smaller" => some(Em(0.8)),
      ~"larger" => some(Em(1.25)),
      ~"inherit" => some(Em(1.0)),
      _  => parse_unit(str),
    }
}

// For width / height, and anything else with the same attribute values
fn parse_size(str : ~str) -> option<Unit> {
    match str {
      ~"auto" => some(Auto),
      ~"inherit" => some(Em(1.0)),
      _ => parse_unit(str),
    }
}

fn parse_display_type(str : ~str) -> option<DisplayType> {
    match str {
      ~"inline" => some(DisInline),
      ~"block" => some(DisBlock),
      ~"none" => some(DisNone),
      _ => { #debug["Recieved unknown display value '%s'", str]; none }
    }
}

#[cfg(test)]
mod test {
    import css_lexer::spawn_css_lexer_from_string;
    import css_builder::build_stylesheet;
    
    #[test]
    fn should_match_font_sizes() {
        let input = ~"* {font-size:12pt; font-size:inherit; font-size:2em; font-size:x-small}";
        let token_port = spawn_css_lexer_from_string(input);
        let actual_rule = build_stylesheet(token_port);
        let expected_rule : Stylesheet = ~[~(~[~Element(~"*", ~[])],
                                             ~[FontSize(Pt(12.0)),
                                               FontSize(Em(1.0)),
                                               FontSize(Em(2.0)),
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
                                               Width(In(3.0)),
                                               Height(Mm(70.0)),
                                               Height(Cm(3.0))])];

        assert actual_rule == expected_rule;
    }
}
