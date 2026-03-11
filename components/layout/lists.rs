/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::counter_style::{CounterStyle, Symbol, SymbolsType};
use style::properties::longhands::list_style_type::computed_value::T as ListStyleType;
use style::values::computed::Image;
use stylo_atoms::atom;

use crate::context::LayoutContext;
use crate::dom_traversal::{NodeAndStyleInfo, PseudoElementContentItem};
use crate::replaced::ReplacedContents;

/// <https://drafts.csswg.org/css-lists/#content-property>
pub(crate) fn make_marker<'dom>(
    context: &LayoutContext,
    info: &NodeAndStyleInfo<'dom>,
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
        Image::CrossFade(..) |
        Image::PaintWorklet(..) |
        Image::None => None,
        Image::LightDark(..) => unreachable!("light-dark() should be disabled"),
    };
    let content = marker_image().or_else(|| {
        Some(vec![PseudoElementContentItem::Text(marker_string(
            &list_style.list_style_type,
        )?)])
    })?;

    Some((marker_info, content))
}

fn symbol_to_string(symbol: &Symbol) -> &str {
    match symbol {
        Symbol::String(string) => string,
        Symbol::Ident(ident) => &ident.0,
    }
}

/// <https://drafts.csswg.org/css-counter-styles-3/#generate-a-counter>
pub(crate) fn generate_counter_representation(counter_style: &CounterStyle) -> &str {
    // TODO: Most counter styles produce different results depending on the counter value.
    // Since we don't support counter properties yet, assume a value of 0 for now.
    match counter_style {
        CounterStyle::None | CounterStyle::String(_) => unreachable!("Invalid counter style"),
        CounterStyle::Name(name) => match name.0 {
            atom!("disc") => "\u{2022}",            /* "•" */
            atom!("circle") => "\u{25E6}",          /* "◦" */
            atom!("square") => "\u{25AA}",          /* "▪" */
            atom!("disclosure-open") => "\u{25BE}", /* "▾" */
            // TODO: Use U+25C2 "◂" depending on the direction.
            atom!("disclosure-closed") => "\u{25B8}", /* "▸" */
            atom!("decimal-leading-zero") => "00",
            atom!("arabic-indic") => "\u{660}", /* "٠" */
            atom!("bengali") => "\u{9E6}",      /* "০" */
            atom!("cambodian") | atom!("khmer") => "\u{17E0}", /* "០" */
            atom!("devanagari") => "\u{966}",   /* "०" */
            atom!("gujarati") => "\u{AE6}",     /* "૦" */
            atom!("gurmukhi") => "\u{A66}",     /* "੦" */
            atom!("kannada") => "\u{CE6}",      /* "೦" */
            atom!("lao") => "\u{ED0}",          /* "໐" */
            atom!("malayalam") => "\u{D66}",    /* "൦" */
            atom!("mongolian") => "\u{1810}",   /* "᠐" */
            atom!("myanmar") => "\u{1040}",     /* "၀" */
            atom!("oriya") => "\u{B66}",        /* "୦" */
            atom!("persian") => "\u{6F0}",      /* "۰" */
            atom!("tamil") => "\u{BE6}",        /* "௦" */
            atom!("telugu") => "\u{C66}",       /* "౦" */
            atom!("thai") => "\u{E50}",         /* "๐" */
            atom!("tibetan") => "\u{F20}",      /* "༠" */
            atom!("cjk-decimal") |
            atom!("cjk-earthly-branch") |
            atom!("cjk-heavenly-stem") |
            atom!("japanese-informal") => "\u{3007}", /* "〇" */
            atom!("korean-hangul-formal") => "\u{C601}", /* "영" */
            atom!("korean-hanja-informal") |
            atom!("korean-hanja-formal") |
            atom!("japanese-formal") |
            atom!("simp-chinese-informal") |
            atom!("simp-chinese-formal") |
            atom!("trad-chinese-informal") |
            atom!("trad-chinese-formal") |
            atom!("cjk-ideographic") => "\u{96F6}", /* "零" */
            // Fall back to decimal.
            _ => "0",
        },
        CounterStyle::Symbols { ty, symbols } => match ty {
            // For numeric, use the first symbol, which represents the value 0.
            SymbolsType::Numeric => {
                symbol_to_string(symbols.0.first().expect("symbols() should have symbols"))
            },
            // For cyclic, the first symbol represents the value 1. However, it loops back,
            // so the last symbol represents the value 0.
            SymbolsType::Cyclic => {
                symbol_to_string(symbols.0.last().expect("symbols() should have symbols"))
            },
            // For the others, the first symbol represents the value 1, and 0 is out of range.
            // Therefore, fall back to `decimal`.
            SymbolsType::Alphabetic | SymbolsType::Symbolic | SymbolsType::Fixed => "0",
        },
    }
}

/// <https://drafts.csswg.org/css-lists/#marker-string>
pub(crate) fn marker_string(list_style_type: &ListStyleType) -> Option<String> {
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
    Some(generate_counter_representation(&list_style_type.0).to_string() + suffix)
}
