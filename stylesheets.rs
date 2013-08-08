use std::iterator::Iterator;
use cssparser::*;
use selectors;
use properties;


struct Stylesheet {
    style_rules: ~[StyleRule],
}


struct StyleRule {
    selectors: ~[selectors::Selector],
    declarations: ~[properties::PropertyDeclaration],
}


fn parse_stylesheet(css: &str) -> Stylesheet {
    let mut rules = ~[];
    for rule in ErrorLogger(parse_stylesheet_rules(tokenize(css))) {
        match rule {
            AtRule(rule) => {
                log_css_error(fmt!("Unsupported at-rule: @%s", rule.name))
            },
            QualifiedRule(rule) => {
                match selectors::parse_selector_list(rule.prelude) {
                    Some(selectors) => rules.push(StyleRule{
                        selectors: selectors,
                        declarations: properties::parse_property_declaration_list(rule.block)
                    }),
                    None => log_css_error("Unsupported CSS selector."),
                }
            },
        }
    }
    Stylesheet{ style_rules: rules }
}


struct ErrorLogger<I>(I);

impl<T, I: Iterator<Result<T, ErrorReason>>> Iterator<T> for ErrorLogger<I> {
    fn next(&mut self) -> Option<T> {
        for result in **self {
            match result {
                Ok(v) => return Some(v),
                Err(e) => log_css_error(fmt!("%?", e))
            }
        }
        None
    }
}


fn log_css_error(message: &str) {
    // TODO eventually this will got into a "web console" or something.
    info!(message)
}
