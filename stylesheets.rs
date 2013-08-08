use std::iterator::Iterator;
use std::hashmap::HashMap;
use std::ascii::to_ascii_lower;
use cssparser::*;
use selectors;
use properties;


pub struct Stylesheet {
    style_rules: ~[StyleRule],
    namespaces: NamespaceMap,
}


pub struct StyleRule {
    selectors: ~[selectors::Selector],
    declarations: ~[properties::PropertyDeclaration],
}

pub struct NamespaceMap {
    default: Option<~str>,  // Optional URL
    prefix_map: HashMap<~str, ~str>,  // prefix -> URL
}


fn parse_stylesheet(css: &str) -> Stylesheet {
    static STATE_CHARSET: uint = 1;
    static STATE_IMPORTS: uint = 2;
    static STATE_NAMESPACES: uint = 3;
    static STATE_BODY: uint = 4;
    let mut state: uint = STATE_CHARSET;

    let mut rules = ~[];
    let mut namespaces = NamespaceMap { default: None, prefix_map: HashMap::new() };

    for rule in ErrorLogger(parse_stylesheet_rules(tokenize(css))) {
        match rule {
            AtRule(rule) => {
                let name = to_ascii_lower(rule.name);
                if "charset" == name {
                    if state > STATE_CHARSET { log_css_error(rule.location,
                        "@charset must be the first rule") }
                    // Valid @charset rules are just ignored
                    loop;
                }
                if "import" == name {
                    if state > STATE_IMPORTS { log_css_error(
                        rule.location, "@import must be before any rule but @charset") }
                    else {
                        state = STATE_IMPORTS;
                        log_css_error(rule.location, "@import is not supported yet")  // TODO
                    }
                    loop;
                }
                if "namespace" == name {
                    if state > STATE_NAMESPACES { log_css_error(rule.location,
                        "@namespace must be before any rule but @charset and @import") }
                    else {
                        state = STATE_NAMESPACES;
                        let location = rule.location;
                        if !parse_namespace_rule(rule, &mut namespaces) {
                            log_css_error(location, "Invalid @namespace rule")
                        }
                    }
                    loop;
                }
                state = STATE_BODY;
                log_css_error(rule.location, fmt!("Unsupported at-rule: @%s", name))
            },
            QualifiedRule(QualifiedRule{location: location, prelude: prelude, block: block}) => {
                state = STATE_BODY;
                match selectors::parse_selector_list(prelude, &namespaces) {
                    Some(selectors) => rules.push(StyleRule{
                        selectors: selectors,
                        declarations: properties::parse_property_declaration_list(block)
                    }),
                    None => log_css_error(location, "Unsupported CSS selector."),
                }
            },
        }
    }
    Stylesheet{ style_rules: rules, namespaces: namespaces }
}


fn parse_namespace_rule(rule: AtRule, namespaces: &mut NamespaceMap) -> bool {
    if rule.block.is_some() { return false }
    let location = rule.location;
    let mut prefix: Option<~str> = None;
    let mut url: Option<~str> = None;
    let mut iter = rule.prelude.consume_skip_whitespace();
    for component_value in iter {
        match component_value {
            Ident(value) => {
                if prefix.is_some() { return false }
                prefix = Some(value);
            },
            URL(value) | String(value) => {
                if url.is_some() { return false }
                url = Some(value);
                break
            },
            _ => return false,
        }
    }
    if iter.next().is_some() { return false }
    match (prefix, url) {
        (Some(prefix), Some(url)) => {
            if namespaces.prefix_map.swap(prefix, url).is_some() {
                log_css_error(location, "Duplicate @namespace rule");
            }
        },
        (None, Some(url)) => {
            if namespaces.default.is_some() {
                log_css_error(location, "Duplicate @namespace rule");
            }
            namespaces.default = Some(url);
        },
        _ => return false
    }
    return true
}


struct ErrorLogger<I>(I);

impl<T, I: Iterator<Result<T, SyntaxError>>> Iterator<T> for ErrorLogger<I> {
    fn next(&mut self) -> Option<T> {
        for result in **self {
            match result {
                Ok(v) => return Some(v),
                Err(error) => log_css_error(error.location, fmt!("%?", error.reason))
            }
        }
        None
    }
}


fn log_css_error(location: SourceLocation, message: &str) {
    // TODO eventually this will got into a "web console" or something.
    info!("%u:%u %s", location.line, location.column, message)
}
