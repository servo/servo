/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::RangeInclusive;

use layout_api::{LayoutElement, LayoutNode};
use script::layout_dom::{ServoLayoutElement, ServoLayoutNode};
use style::counter_style::{CounterStyle, Symbol, SymbolsType};
use style::properties::longhands::list_style_type::computed_value::T as ListStyleType;
use style::values::computed::Image;
use style::values::generics::counters::Content;
use stylo_atoms::{Atom, atom};
use web_atoms::{LocalName, local_name, ns};

use crate::context::LayoutContext;
use crate::dom_traversal::{
    NodeAndStyleInfo, PseudoElementContentItem, generate_pseudo_element_content,
};
use crate::replaced::ReplacedContents;

/// <https://drafts.csswg.org/css-lists/#content-property>
pub(crate) fn make_marker<'dom>(
    context: &LayoutContext,
    info: &NodeAndStyleInfo<'dom>,
    ordinal: i32,
) -> Option<(NodeAndStyleInfo<'dom>, Vec<PseudoElementContentItem>)> {
    let marker_info =
        info.with_pseudo_element(context, style::selector_parser::PseudoElement::Marker)?;
    let style = &marker_info.style;
    let list_style = style.get_list();

    // https://drafts.csswg.org/css-lists/#marker-image
    let marker_image = || match &list_style.list_style_image {
        Image::Url(url) => Some(vec![
            PseudoElementContentItem::Replaced(ReplacedContents::from_image_url(
                marker_info.node,
                context,
                url,
            )?),
            PseudoElementContentItem::Text(" ".into()),
        ]),
        // XXX: Non-None image types unimplemented.
        Image::ImageSet(..) |
        Image::Gradient(..) |
        Image::Image(..) |
        Image::CrossFade(..) |
        Image::PaintWorklet(..) |
        Image::None => None,
        Image::LightDark(..) => unreachable!("light-dark() should be disabled"),
    };

    let content = match &marker_info.style.get_counters().content {
        Content::Items(_) => generate_pseudo_element_content(&marker_info, context),
        Content::None => return None,
        Content::Normal => marker_image().or_else(|| {
            Some(vec![PseudoElementContentItem::Text(marker_string(
                &list_style.list_style_type,
                ordinal,
            )?)])
        })?,
    };

    Some((marker_info, content))
}

/// The numbers a list gives to the items in it, in the order they are laid out.
///
/// Servo does not implement CSS counters, so rather than reading `counter-reset` and
/// `counter-increment` this follows the HTML rules for an ordinal value, which is what
/// the user agent stylesheet would map those attributes to.
/// <https://html.spec.whatwg.org/multipage/#ordinal-value>
pub(crate) struct ListNumbering {
    next: i32,
    step: i32,
}

impl ListNumbering {
    pub(crate) fn of_list(node: ServoLayoutNode) -> Self {
        let Some(element) = node
            .as_html_element()
            .filter(|element| *element.local_name() == local_name!("ol"))
        else {
            return Self { next: 1, step: 1 };
        };

        let start = integer_attribute(&element, &local_name!("start"));
        match element
            .attribute(&ns!(), &local_name!("reversed"))
            .is_some()
        {
            true => Self {
                next: start.unwrap_or_else(|| reversed_start(node)),
                step: -1,
            },
            false => Self {
                next: start.unwrap_or(1),
                step: 1,
            },
        }
    }

    /// The number for the next item, which an item may choose for itself.
    pub(crate) fn next(&mut self, item: ServoLayoutNode) -> i32 {
        let chosen = item
            .as_html_element()
            .and_then(|element| integer_attribute(&element, &local_name!("value")));
        let ordinal = chosen.unwrap_or(self.next);
        self.next = ordinal.saturating_add(self.step);
        ordinal
    }
}

fn integer_attribute(element: &ServoLayoutElement, name: &LocalName) -> Option<i32> {
    element.attribute_as_str(&ns!(), name)?.trim().parse().ok()
}

/// Where a reversed list starts counting down from, which is far enough above the first
/// item that chooses its own number for the list to reach that number, or the number of
/// items when none of them chooses.
/// <https://html.spec.whatwg.org/multipage/#attr-ol-reversed>
fn reversed_start(node: ServoLayoutNode) -> i32 {
    let mut items_before = 0;
    for child in node.flat_tree_children() {
        let Some(item) = child
            .as_html_element()
            .filter(|element| *element.local_name() == local_name!("li"))
        else {
            continue;
        };
        if let Some(chosen) = integer_attribute(&item, &local_name!("value")) {
            return chosen.saturating_add(items_before);
        }
        items_before += 1;
    }
    items_before
}

fn symbol_to_string(symbol: &Symbol) -> &str {
    match symbol {
        Symbol::String(string) => string,
        Symbol::Ident(ident) => &ident.0,
    }
}

/// The ten digits of a counter style that writes its value positionally.
/// <https://drafts.csswg.org/css-counter-styles-3/#numeric-system>
fn numeric_digits(name: &Atom) -> Option<&'static str> {
    Some(match *name {
        atom!("decimal") | atom!("decimal-leading-zero") => "0123456789",
        atom!("arabic-indic") => "٠١٢٣٤٥٦٧٨٩",
        atom!("bengali") => "০১২৩৪৫৬৭৮৯",
        atom!("cambodian") | atom!("khmer") => "០១២៣៤៥៦៧៨៩",
        atom!("devanagari") => "०१२३४५६७८९",
        atom!("gujarati") => "૦૧૨૩૪૫૬૭૮૯",
        atom!("gurmukhi") => "੦੧੨੩੪੫੬੭੮੯",
        atom!("kannada") => "೦೧೨೩೪೫೬೭೮೯",
        atom!("lao") => "໐໑໒໓໔໕໖໗໘໙",
        atom!("malayalam") => "൦൧൨൩൪൫൬൭൮൯",
        atom!("mongolian") => "᠐᠑᠒᠓᠔᠕᠖᠗᠘᠙",
        atom!("myanmar") => "၀၁၂၃၄၅၆၇၈၉",
        atom!("oriya") => "୦୧୨୩୪୫୬୭୮୯",
        atom!("persian") => "۰۱۲۳۴۵۶۷۸۹",
        atom!("tamil") => "௦௧௨௩௪௫௬௭௮௯",
        atom!("telugu") => "౦౧౨౩౪౫౬౭౮౯",
        atom!("thai") => "๐๑๒๓๔๕๖๗๘๙",
        atom!("tibetan") => "༠༡༢༣༤༥༦༧༨༩",
        atom!("cjk-decimal") => "〇一二三四五六七八九",
        _ => return None,
    })
}

/// The letters of a counter style that counts through them, then through pairs of them.
/// <https://drafts.csswg.org/css-counter-styles-3/#alphabetic-system>
fn alphabetic_letters(name: &Atom) -> Option<&'static str> {
    Some(match *name {
        atom!("lower-alpha") | atom!("lower-latin") => "abcdefghijklmnopqrstuvwxyz",
        atom!("upper-alpha") | atom!("upper-latin") => "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        atom!("lower-greek") => "αβγδεζηθικλμνξοπρστυφχψω",
        _ => return None,
    })
}

/// The symbol a counter style shows whatever its value is.
/// <https://drafts.csswg.org/css-counter-styles-3/#cyclic-system>
fn cyclic_symbol(name: &Atom) -> Option<&'static str> {
    Some(match *name {
        atom!("disc") => "\u{2022}",            /* "•" */
        atom!("circle") => "\u{25E6}",          /* "◦" */
        atom!("square") => "\u{25AA}",          /* "▪" */
        atom!("disclosure-open") => "\u{25BE}", /* "▾" */
        // TODO: Use U+25C2 "◂" depending on the direction.
        atom!("disclosure-closed") => "\u{25B8}", /* "▸" */
        _ => return None,
    })
}

/// A counter style that writes a value by adding symbols together.
/// <https://drafts.csswg.org/css-counter-styles-3/#additive-system>
struct Additive {
    /// What each symbol is worth, heaviest first.
    symbols: &'static [(u32, &'static str)],
    /// The values the style is defined for.
    range: RangeInclusive<i32>,
}

fn additive_symbols(name: &Atom) -> Option<Additive> {
    const UPPER_ROMAN: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    const LOWER_ROMAN: &[(u32, &str)] = &[
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];

    // <https://drafts.csswg.org/css-counter-styles-3/#simple-numeric> gives both roman
    // styles a range of 1 to 3999.
    let symbols = match *name {
        atom!("lower-roman") => LOWER_ROMAN,
        atom!("upper-roman") => UPPER_ROMAN,
        _ => return None,
    };
    Some(Additive {
        symbols,
        range: 1..=3999,
    })
}

/// Write the value in the given digits, padded out to at least `least_digits` of them.
fn positional(value: i32, digits: &str, least_digits: usize) -> String {
    let digits: Vec<char> = digits.chars().collect();
    let base = digits.len() as u32;
    let mut magnitude = value.unsigned_abs();
    let mut written = Vec::new();

    while magnitude > 0 || written.len() < least_digits {
        written.push(digits[(magnitude % base) as usize]);
        magnitude /= base;
    }
    let sign = if value < 0 { "-" } else { "" };
    let written: String = written.iter().rev().collect();
    format!("{sign}{written}")
}

/// Count a, b, …, z, aa, ab, … Only defined for positive values.
fn alphabetic(value: i32, letters: &str) -> Option<String> {
    let letters: Vec<char> = letters.chars().collect();
    let base = letters.len() as u32;
    let mut remaining = u32::try_from(value).ok().filter(|value| *value > 0)?;
    let mut written = Vec::new();

    while remaining > 0 {
        remaining -= 1;
        written.push(letters[(remaining % base) as usize]);
        remaining /= base;
    }
    Some(written.iter().rev().collect())
}

/// Add symbols together until they make up the value, which the style has to be defined
/// for and the symbols have to reach exactly.
fn additive(value: i32, style: &Additive) -> Option<String> {
    let mut remaining = u32::try_from(value)
        .ok()
        .filter(|_| style.range.contains(&value))?;
    let mut written = String::new();

    for (weight, symbol) in style.symbols {
        while remaining >= *weight {
            written.push_str(symbol);
            remaining -= weight;
        }
    }
    (remaining == 0).then_some(written)
}

/// Repeat a symbol once for every time the value goes round the list.
/// <https://drafts.csswg.org/css-counter-styles-3/#symbolic-system>
fn symbolic(value: i32, symbols: &[Symbol]) -> Option<String> {
    let value = usize::try_from(value).ok().filter(|value| *value > 0)?;
    let symbol = symbol_to_string(&symbols[(value - 1) % symbols.len()]);
    Some(symbol.repeat(value.div_ceil(symbols.len())))
}

/// <https://drafts.csswg.org/css-counter-styles-3/#generate-a-counter>
///
/// A style that cannot write a value falls back to `decimal`, as the specification asks
/// for when a value is outside a style's range. A style this does not implement at all
/// falls back the same way.
pub(crate) fn generate_counter_representation(counter_style: &CounterStyle, value: i32) -> String {
    match counter_style {
        CounterStyle::None | CounterStyle::String(_) => unreachable!("Invalid counter style"),
        CounterStyle::Name(name) => representation_for_name(&name.0, value),
        CounterStyle::Symbols { ty, symbols } => representation_for_symbols(*ty, &symbols.0, value),
    }
}

fn representation_for_name(name: &Atom, value: i32) -> String {
    if let Some(symbol) = cyclic_symbol(name) {
        return symbol.to_owned();
    }
    if let Some(digits) = numeric_digits(name) {
        let least_digits = match *name {
            atom!("decimal-leading-zero") => 2,
            _ => 1,
        };
        return positional(value, digits, least_digits);
    }
    if let Some(letters) = alphabetic_letters(name) {
        return alphabetic(value, letters).unwrap_or_else(|| value.to_string());
    }
    if let Some(style) = additive_symbols(name) {
        return additive(value, &style).unwrap_or_else(|| value.to_string());
    }
    value.to_string()
}

fn representation_for_symbols(ty: SymbolsType, symbols: &[Symbol], value: i32) -> String {
    let nth = |index: usize| symbol_to_string(&symbols[index]).to_owned();
    let letters: String = symbols.iter().map(symbol_to_string).collect();

    match ty {
        SymbolsType::Cyclic => nth(value.rem_euclid(symbols.len() as i32) as usize),
        SymbolsType::Numeric => positional(value, &letters, 1),
        SymbolsType::Alphabetic => alphabetic(value, &letters).unwrap_or_else(|| value.to_string()),
        SymbolsType::Symbolic => symbolic(value, symbols).unwrap_or_else(|| value.to_string()),
        // The first symbol stands for the value 1, and anything past the last symbol is
        // written with the fallback style.
        SymbolsType::Fixed => match usize::try_from(value).ok().filter(|value| *value > 0) {
            Some(value) if value <= symbols.len() => nth(value - 1),
            _ => value.to_string(),
        },
    }
}

/// <https://drafts.csswg.org/css-lists/#marker-string>
pub(crate) fn marker_string(list_style_type: &ListStyleType, value: i32) -> Option<String> {
    let suffix = match &list_style_type.0 {
        CounterStyle::None => return None,
        CounterStyle::String(string) => return Some(string.to_string()),
        CounterStyle::Name(name) => match name.0 {
            atom!("disc") |
            atom!("circle") |
            atom!("square") |
            atom!("disclosure-open") |
            atom!("disclosure-closed") => " ",
            atom!("hiragana") |
            atom!("hiragana-iroha") |
            atom!("katakana") |
            atom!("katakana-iroha") |
            atom!("cjk-decimal") |
            atom!("cjk-earthly-branch") |
            atom!("cjk-heavenly-stem") |
            atom!("japanese-informal") |
            atom!("japanese-formal") |
            atom!("simp-chinese-informal") |
            atom!("simp-chinese-formal") |
            atom!("trad-chinese-informal") |
            atom!("trad-chinese-formal") |
            atom!("cjk-ideographic") => "\u{3001}", /* "、" */
            atom!("korean-hangul-formal") |
            atom!("korean-hanja-informal") |
            atom!("korean-hanja-formal") => ", ",
            atom!("ethiopic-numeric") => "/ ",
            _ => ". ",
        },
        CounterStyle::Symbols { .. } => " ",
    };
    Some(format!(
        "{}{}",
        generate_counter_representation(&list_style_type.0, value),
        suffix
    ))
}

#[cfg(test)]
mod tests {
    use stylo_atoms::Atom;

    use super::representation_for_name;

    fn representation(style: &str, value: i32) -> String {
        representation_for_name(&Atom::from(style), value)
    }

    #[test]
    fn decimal_counts_up() {
        assert_eq!(representation("decimal", 1), "1");
        assert_eq!(representation("decimal", 42), "42");
        assert_eq!(representation("decimal", 0), "0");
        assert_eq!(representation("decimal", -7), "-7");
    }

    #[test]
    fn decimal_leading_zero_pads_to_two_digits() {
        assert_eq!(representation("decimal-leading-zero", 1), "01");
        assert_eq!(representation("decimal-leading-zero", 10), "10");
        assert_eq!(representation("decimal-leading-zero", 100), "100");
    }

    #[test]
    fn other_digits_write_the_same_number() {
        assert_eq!(representation("arabic-indic", 12), "١٢");
        assert_eq!(representation("cjk-decimal", 20), "二〇");
    }

    #[test]
    fn letters_carry_into_a_second_letter() {
        assert_eq!(representation("lower-alpha", 1), "a");
        assert_eq!(representation("lower-alpha", 26), "z");
        assert_eq!(representation("lower-alpha", 27), "aa");
        assert_eq!(representation("upper-alpha", 28), "AB");
    }

    #[test]
    fn roman_numerals_add_up() {
        assert_eq!(representation("upper-roman", 4), "IV");
        assert_eq!(representation("lower-roman", 9), "ix");
        assert_eq!(representation("upper-roman", 1984), "MCMLXXXIV");
    }

    #[test]
    fn a_value_a_style_cannot_write_falls_back_to_decimal() {
        // Letters and roman numerals start at one.
        assert_eq!(representation("lower-alpha", 0), "0");
        assert_eq!(representation("upper-roman", -1), "-1");
        // Roman numerals stop at 3999.
        assert_eq!(representation("upper-roman", 4000), "4000");
    }

    #[test]
    fn a_style_without_numbers_ignores_the_value() {
        assert_eq!(representation("disc", 3), "\u{2022}");
        assert_eq!(representation("square", 9), "\u{25AA}");
    }

    #[test]
    fn an_unimplemented_style_falls_back_to_decimal() {
        assert_eq!(representation("hebrew", 5), "5");
    }
}
