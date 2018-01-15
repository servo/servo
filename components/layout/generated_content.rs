/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The generated content assignment phase.
//!
//! This phase handles CSS counters, quotes, and ordered lists per CSS § 12.3-12.5. It cannot be
//! done in parallel and is therefore a sequential pass that runs on as little of the flow tree
//! as possible.

use context::{LayoutContext, with_thread_local_font_context};
use flow::{Flow, FlowFlags, GetBaseFlow, ImmutableFlowUtils};
use fragment::{Fragment, GeneratedContentInfo, SpecificFragmentInfo, UnscannedTextFragmentInfo};
use gfx::display_list::OpaqueNode;
use script_layout_interface::wrapper_traits::PseudoElementType;
use smallvec::SmallVec;
use std::collections::{HashMap, LinkedList};
use style::computed_values::content::ContentItem;
use style::computed_values::display::T as Display;
use style::computed_values::list_style_type::T as ListStyleType;
use style::properties::ComputedValues;
use style::selector_parser::RestyleDamage;
use style::servo::restyle_damage::ServoRestyleDamage;
use text::TextRunScanner;
use traversal::InorderFlowTraversal;

// Decimal styles per CSS-COUNTER-STYLES § 6.1:
static DECIMAL: [char; 10] = [ '0', '1', '2', '3', '4', '5', '6', '7', '8', '9' ];
// TODO(pcwalton): `decimal-leading-zero`
static ARABIC_INDIC: [char; 10] = [ '٠', '١', '٢', '٣', '٤', '٥', '٦', '٧', '٨', '٩' ];
// TODO(pcwalton): `armenian`, `upper-armenian`, `lower-armenian`
static BENGALI: [char; 10] = [ '০', '১', '২', '৩', '৪', '৫', '৬', '৭', '৮', '৯' ];
static CAMBODIAN: [char; 10] = [ '០', '១', '២', '៣', '៤', '៥', '៦', '៧', '៨', '៩' ];
// TODO(pcwalton): Suffix for CJK decimal.
static CJK_DECIMAL: [char; 10] = [ '〇', '一', '二', '三', '四', '五', '六', '七', '八', '九' ];
static DEVANAGARI: [char; 10] = [ '०', '१', '२', '३', '४', '५', '६', '७', '८', '९' ];
// TODO(pcwalton): `georgian`
static GUJARATI: [char; 10] = ['૦', '૧', '૨', '૩', '૪', '૫', '૬', '૭', '૮', '૯'];
static GURMUKHI: [char; 10] = ['੦', '੧', '੨', '੩', '੪', '੫', '੬', '੭', '੮', '੯'];
// TODO(pcwalton): `hebrew`
static KANNADA: [char; 10] = ['೦', '೧', '೨', '೩', '೪', '೫', '೬', '೭', '೮', '೯'];
static LAO: [char; 10] = ['໐', '໑', '໒', '໓', '໔', '໕', '໖', '໗', '໘', '໙'];
static MALAYALAM: [char; 10] = ['൦', '൧', '൨', '൩', '൪', '൫', '൬', '൭', '൮', '൯'];
static MONGOLIAN: [char; 10] = ['᠐', '᠑', '᠒', '᠓', '᠔', '᠕', '᠖', '᠗', '᠘', '᠙'];
static MYANMAR: [char; 10] = ['၀', '၁', '၂', '၃', '၄', '၅', '၆', '၇', '၈', '၉'];
static ORIYA: [char; 10] = ['୦', '୧', '୨', '୩', '୪', '୫', '୬', '୭', '୮', '୯'];
static PERSIAN: [char; 10] = ['۰', '۱', '۲', '۳', '۴', '۵', '۶', '۷', '۸', '۹'];
// TODO(pcwalton): `lower-roman`, `upper-roman`
static TELUGU: [char; 10] = ['౦', '౧', '౨', '౩', '౪', '౫', '౬', '౭', '౮', '౯'];
static THAI: [char; 10] = ['๐', '๑', '๒', '๓', '๔', '๕', '๖', '๗', '๘', '๙'];
static TIBETAN: [char; 10] = ['༠', '༡', '༢', '༣', '༤', '༥', '༦', '༧', '༨', '༩'];

// Alphabetic styles per CSS-COUNTER-STYLES § 6.2:
static LOWER_ALPHA: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z'
];
static UPPER_ALPHA: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z'
];
static CJK_EARTHLY_BRANCH: [char; 12] = [
    '子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'
];
static CJK_HEAVENLY_STEM: [char; 10] = [
    '甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'
];
static LOWER_GREEK: [char; 24] = [
    'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν', 'ξ', 'ο', 'π', 'ρ', 'σ', 'τ',
    'υ', 'φ', 'χ', 'ψ', 'ω'
];
static HIRAGANA: [char; 48] = [
    'あ', 'い', 'う', 'え', 'お', 'か', 'き', 'く', 'け', 'こ', 'さ', 'し', 'す', 'せ', 'そ',
    'た', 'ち', 'つ', 'て', 'と', 'な', 'に', 'ぬ', 'ね', 'の', 'は', 'ひ', 'ふ', 'へ', 'ほ',
    'ま', 'み', 'む', 'め', 'も', 'や', 'ゆ', 'よ', 'ら', 'り', 'る', 'れ', 'ろ',
    'わ', 'ゐ', 'ゑ', 'を', 'ん'
];
static HIRAGANA_IROHA: [char; 47] = [
    'い', 'ろ', 'は', 'に', 'ほ', 'へ', 'と', 'ち', 'り', 'ぬ', 'る', 'を', 'わ', 'か', 'よ',
    'た', 'れ', 'そ', 'つ', 'ね', 'な', 'ら', 'む', 'う', 'ゐ', 'の', 'お', 'く', 'や', 'ま',
    'け', 'ふ', 'こ', 'え', 'て', 'あ', 'さ', 'き', 'ゆ', 'め', 'み', 'し', 'ゑ',
    'ひ', 'も', 'せ', 'す'
];
static KATAKANA: [char; 48] = [
    'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ',
    'タ', 'チ', 'ツ', 'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ', 'フ', 'ヘ', 'ホ',
    'マ', 'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル', 'レ', 'ロ',
    'ワ', 'ヰ', 'ヱ', 'ヲ', 'ン'
];
static KATAKANA_IROHA: [char; 47] = [
    'イ', 'ロ', 'ハ', 'ニ', 'ホ', 'ヘ', 'ト', 'チ', 'リ', 'ヌ', 'ル', 'ヲ', 'ワ', 'カ', 'ヨ',
    'タ', 'レ', 'ソ', 'ツ', 'ネ', 'ナ', 'ラ', 'ム', 'ウ', 'ヰ', 'ノ', 'オ', 'ク', 'ヤ', 'マ',
    'ケ', 'フ', 'コ', 'エ', 'テ', 'ア', 'サ', 'キ', 'ユ', 'メ', 'ミ', 'シ', 'ヱ',
    'ヒ', 'モ', 'セ', 'ス'
];

/// The generated content resolution traversal.
pub struct ResolveGeneratedContent<'a> {
    /// The layout context.
    layout_context: &'a LayoutContext<'a>,
    /// The counter representing an ordered list item.
    list_item: Counter,
    /// Named CSS counters.
    counters: HashMap<String, Counter>,
    /// The level of quote nesting.
    quote: u32,
}

impl<'a> ResolveGeneratedContent<'a> {
    /// Creates a new generated content resolution traversal.
    pub fn new(layout_context: &'a LayoutContext) -> ResolveGeneratedContent<'a> {
        ResolveGeneratedContent {
            layout_context: layout_context,
            list_item: Counter::new(),
            counters: HashMap::new(),
            quote: 0,
        }
    }
}

impl<'a> InorderFlowTraversal for ResolveGeneratedContent<'a> {
    #[inline]
    fn process(&mut self, flow: &mut Flow, level: u32) {
        let mut mutator = ResolveGeneratedContentFragmentMutator {
            traversal: self,
            level: level,
            is_block: flow.is_block_like(),
            incremented: false,
        };
        flow.mutate_fragments(&mut |fragment| mutator.mutate_fragment(fragment))
    }

    #[inline]
    fn should_process_subtree(&mut self, flow: &mut Flow) -> bool {
        flow.base().restyle_damage.intersects(ServoRestyleDamage::RESOLVE_GENERATED_CONTENT) ||
            flow.base().flags.intersects(FlowFlags::AFFECTS_COUNTERS | FlowFlags::HAS_COUNTER_AFFECTING_CHILDREN)
    }
}

/// The object that mutates the generated content fragments.
struct ResolveGeneratedContentFragmentMutator<'a, 'b: 'a> {
    /// The traversal.
    traversal: &'a mut ResolveGeneratedContent<'b>,
    /// The level we're at in the flow tree.
    level: u32,
    /// Whether this flow is a block flow.
    is_block: bool,
    /// Whether we've incremented the counter yet.
    incremented: bool,
}

impl<'a, 'b> ResolveGeneratedContentFragmentMutator<'a, 'b> {
    fn mutate_fragment(&mut self, fragment: &mut Fragment) {
        // We only reset and/or increment counters once per flow. This avoids double-incrementing
        // counters on list items (once for the main fragment and once for the marker).
        if !self.incremented {
            self.reset_and_increment_counters_as_necessary(fragment);
        }

        let mut list_style_type = fragment.style().get_list().list_style_type;
        if fragment.style().get_box().display != Display::ListItem {
            list_style_type = ListStyleType::None
        }

        let mut new_info = None;
        {
            let info =
                if let SpecificFragmentInfo::GeneratedContent(ref mut info) = fragment.specific {
                    info
                } else {
                    return
                };

            match **info {
                GeneratedContentInfo::ListItem => {
                    new_info = self.traversal.list_item.render(self.traversal.layout_context,
                                                               fragment.node,
                                                               fragment.pseudo.clone(),
                                                               fragment.style.clone(),
                                                               list_style_type,
                                                               RenderingMode::Suffix(".\u{00a0}"))
                }
                GeneratedContentInfo::Empty |
                GeneratedContentInfo::ContentItem(ContentItem::String(_)) => {
                    // Nothing to do here.
                }
                GeneratedContentInfo::ContentItem(ContentItem::Counter(ref counter_name,
                                                                       counter_style)) => {
                    let temporary_counter = Counter::new();
                    let counter = self.traversal
                                      .counters
                                      .get(&*counter_name)
                                      .unwrap_or(&temporary_counter);
                    new_info = counter.render(self.traversal.layout_context,
                                              fragment.node,
                                              fragment.pseudo.clone(),
                                              fragment.style.clone(),
                                              counter_style,
                                              RenderingMode::Plain)
                }
                GeneratedContentInfo::ContentItem(ContentItem::Counters(ref counter_name,
                                                                        ref separator,
                                                                        counter_style)) => {
                    let temporary_counter = Counter::new();
                    let counter = self.traversal
                                      .counters
                                      .get(&*counter_name)
                                      .unwrap_or(&temporary_counter);
                    new_info = counter.render(self.traversal.layout_context,
                                              fragment.node,
                                              fragment.pseudo,
                                              fragment.style.clone(),
                                              counter_style,
                                              RenderingMode::All(&separator));
                }
                GeneratedContentInfo::ContentItem(ContentItem::OpenQuote) => {
                    new_info = render_text(self.traversal.layout_context,
                                           fragment.node,
                                           fragment.pseudo,
                                           fragment.style.clone(),
                                           self.quote(&*fragment.style, false));
                    self.traversal.quote += 1
                }
                GeneratedContentInfo::ContentItem(ContentItem::CloseQuote) => {
                    if self.traversal.quote >= 1 {
                        self.traversal.quote -= 1
                    }

                    new_info = render_text(self.traversal.layout_context,
                                           fragment.node,
                                           fragment.pseudo,
                                           fragment.style.clone(),
                                           self.quote(&*fragment.style, true));
                }
                GeneratedContentInfo::ContentItem(ContentItem::NoOpenQuote) => {
                    self.traversal.quote += 1
                }
                GeneratedContentInfo::ContentItem(ContentItem::NoCloseQuote) => {
                    if self.traversal.quote >= 1 {
                        self.traversal.quote -= 1
                    }
                }
            }
        };

        fragment.specific = match new_info {
            Some(new_info) => new_info,
            // If the fragment did not generate any content, replace it with a no-op placeholder
            // so that it isn't processed again on the next layout.  FIXME (mbrubeck): When
            // processing an inline flow, this traversal should be allowed to insert or remove
            // fragments.  Then we can just remove these fragments rather than adding placeholders.
            None => SpecificFragmentInfo::GeneratedContent(Box::new(GeneratedContentInfo::Empty))
        };
    }

    fn reset_and_increment_counters_as_necessary(&mut self, fragment: &mut Fragment) {
        let mut list_style_type = fragment.style().get_list().list_style_type;
        if !self.is_block || fragment.style().get_box().display != Display::ListItem {
            list_style_type = ListStyleType::None
        }

        match list_style_type {
            ListStyleType::Disc | ListStyleType::None | ListStyleType::Circle |
            ListStyleType::Square | ListStyleType::DisclosureOpen |
            ListStyleType::DisclosureClosed => {}
            _ => self.traversal.list_item.increment(self.level, 1),
        }

        // Truncate down counters.
        for (_, counter) in &mut self.traversal.counters {
            counter.truncate_to_level(self.level);
        }
        self.traversal.list_item.truncate_to_level(self.level);

        for &(ref counter_name, value) in &fragment.style().get_counters().counter_reset.0 {
            let counter_name = &*counter_name.0;
            if let Some(ref mut counter) = self.traversal.counters.get_mut(counter_name) {
                 counter.reset(self.level, value);
                 continue
            }

            let mut counter = Counter::new();
            counter.reset(self.level, value);
            self.traversal.counters.insert(counter_name.to_owned(), counter);
        }

        for &(ref counter_name, value) in &fragment.style().get_counters().counter_increment.0 {
            let counter_name = &*counter_name.0;
            if let Some(ref mut counter) = self.traversal.counters.get_mut(counter_name) {
                counter.increment(self.level, value);
                continue
            }

            let mut counter = Counter::new();
            counter.increment(self.level, value);
            self.traversal.counters.insert(counter_name.to_owned(), counter);
        }

        self.incremented = true
    }

    fn quote(&self, style: &ComputedValues, close: bool) -> String {
        let quotes = &style.get_list().quotes;
        if quotes.0.is_empty() {
            return String::new()
        }
        let &(ref open_quote, ref close_quote) =
            if self.traversal.quote as usize >= quotes.0.len() {
                quotes.0.last().unwrap()
            } else {
                &quotes.0[self.traversal.quote as usize]
            };
        if close {
            close_quote.to_string()
        } else {
            open_quote.to_string()
        }
    }
}

/// A counter per CSS 2.1 § 12.4.
struct Counter {
    /// The values at each level.
    values: Vec<CounterValue>,
}

impl Counter {
    fn new() -> Counter {
        Counter {
            values: Vec::new(),
        }
    }

    fn reset(&mut self, level: u32, value: i32) {
        // Do we have an instance of the counter at this level? If so, just mutate it.
        if let Some(ref mut existing_value) = self.values.last_mut() {
            if level == existing_value.level {
                existing_value.value = value;
                return
            }
        }

        // Otherwise, push a new instance of the counter.
        self.values.push(CounterValue {
            level: level,
            value: value,
        })
    }

    fn truncate_to_level(&mut self, level: u32) {
        if let Some(position) = self.values.iter().position(|value| value.level > level) {
            self.values.truncate(position)
        }
    }

    fn increment(&mut self, level: u32, amount: i32) {
        if let Some(ref mut value) = self.values.last_mut() {
            value.value += amount;
            return
        }

        self.values.push(CounterValue {
            level: level,
            value: amount,
        })
    }

    fn render(&self,
              layout_context: &LayoutContext,
              node: OpaqueNode,
              pseudo: PseudoElementType,
              style: ::ServoArc<ComputedValues>,
              list_style_type: ListStyleType,
              mode: RenderingMode)
              -> Option<SpecificFragmentInfo> {
        let mut string = String::new();
        match mode {
            RenderingMode::Plain => {
                let value = match self.values.last() {
                    Some(ref value) => value.value,
                    None => 0,
                };
                push_representation(value, list_style_type, &mut string)
            }
            RenderingMode::Suffix(suffix) => {
                let value = match self.values.last() {
                    Some(ref value) => value.value,
                    None => 0,
                };
                push_representation(value, list_style_type, &mut string);
                string.push_str(suffix)
            }
            RenderingMode::All(separator) => {
                let mut first = true;
                for value in &self.values {
                    if !first {
                        string.push_str(separator)
                    }
                    first = false;
                    push_representation(value.value, list_style_type, &mut string)
                }
            }
        }

        if string.is_empty() {
            None
        } else {
            render_text(layout_context, node, pseudo, style, string)
        }
    }
}

/// How a counter value is to be rendered.
enum RenderingMode<'a> {
    /// The innermost counter value is rendered with no extra decoration.
    Plain,
    /// The innermost counter value is rendered with the given string suffix.
    Suffix(&'a str),
    /// All values of the counter are rendered with the given separator string between them.
    All(&'a str),
}

/// The value of a counter at a given level.
struct CounterValue {
    /// The level of the flow tree that this corresponds to.
    level: u32,
    /// The value of the counter at this level.
    value: i32,
}

/// Creates fragment info for a literal string.
fn render_text(layout_context: &LayoutContext,
               node: OpaqueNode,
               pseudo: PseudoElementType,
               style: ::ServoArc<ComputedValues>,
               string: String)
               -> Option<SpecificFragmentInfo> {
    let mut fragments = LinkedList::new();
    let info = SpecificFragmentInfo::UnscannedText(
        Box::new(UnscannedTextFragmentInfo::new(string, None))
    );
    fragments.push_back(Fragment::from_opaque_node_and_style(node,
                                                             pseudo,
                                                             style.clone(),
                                                             style,
                                                             RestyleDamage::rebuild_and_reflow(),
                                                             info));
    // FIXME(pcwalton): This should properly handle multiple marker fragments. This could happen
    // due to text run splitting.
    let fragments = with_thread_local_font_context(layout_context, |font_context| {
        TextRunScanner::new().scan_for_runs(font_context, fragments)
    });
    if fragments.is_empty() {
        None
    } else {
        Some(fragments.fragments.into_iter().next().unwrap().specific)
    }
}

/// Appends string that represents the value rendered using the system appropriate for the given
/// `list-style-type` onto the given string.
fn push_representation(value: i32, list_style_type: ListStyleType, accumulator: &mut String) {
    match list_style_type {
        ListStyleType::None => {}
        ListStyleType::Disc |
        ListStyleType::Circle |
        ListStyleType::Square |
        ListStyleType::DisclosureOpen |
        ListStyleType::DisclosureClosed => {
            accumulator.push(static_representation(list_style_type))
        }
        ListStyleType::Decimal => push_numeric_representation(value, &DECIMAL, accumulator),
        ListStyleType::ArabicIndic => {
            push_numeric_representation(value, &ARABIC_INDIC, accumulator)
        }
        ListStyleType::Bengali => push_numeric_representation(value, &BENGALI, accumulator),
        ListStyleType::Cambodian |
        ListStyleType::Khmer => {
            push_numeric_representation(value, &CAMBODIAN, accumulator)
        }
        ListStyleType::CjkDecimal => {
            push_numeric_representation(value, &CJK_DECIMAL, accumulator)
        }
        ListStyleType::Devanagari => {
            push_numeric_representation(value, &DEVANAGARI, accumulator)
        }
        ListStyleType::Gujarati => push_numeric_representation(value, &GUJARATI, accumulator),
        ListStyleType::Gurmukhi => push_numeric_representation(value, &GURMUKHI, accumulator),
        ListStyleType::Kannada => push_numeric_representation(value, &KANNADA, accumulator),
        ListStyleType::Lao => push_numeric_representation(value, &LAO, accumulator),
        ListStyleType::Malayalam => {
            push_numeric_representation(value, &MALAYALAM, accumulator)
        }
        ListStyleType::Mongolian => {
            push_numeric_representation(value, &MONGOLIAN, accumulator)
        }
        ListStyleType::Myanmar => push_numeric_representation(value, &MYANMAR, accumulator),
        ListStyleType::Oriya => push_numeric_representation(value, &ORIYA, accumulator),
        ListStyleType::Persian => push_numeric_representation(value, &PERSIAN, accumulator),
        ListStyleType::Telugu => push_numeric_representation(value, &TELUGU, accumulator),
        ListStyleType::Thai => push_numeric_representation(value, &THAI, accumulator),
        ListStyleType::Tibetan => push_numeric_representation(value, &TIBETAN, accumulator),
        ListStyleType::LowerAlpha => {
            push_alphabetic_representation(value, &LOWER_ALPHA, accumulator)
        }
        ListStyleType::UpperAlpha => {
            push_alphabetic_representation(value, &UPPER_ALPHA, accumulator)
        }
        ListStyleType::CjkEarthlyBranch => {
            push_alphabetic_representation(value, &CJK_EARTHLY_BRANCH, accumulator)
        }
        ListStyleType::CjkHeavenlyStem => {
            push_alphabetic_representation(value, &CJK_HEAVENLY_STEM, accumulator)
        }
        ListStyleType::LowerGreek => {
            push_alphabetic_representation(value, &LOWER_GREEK, accumulator)
        }
        ListStyleType::Hiragana => {
            push_alphabetic_representation(value, &HIRAGANA, accumulator)
        }
        ListStyleType::HiraganaIroha => {
            push_alphabetic_representation(value, &HIRAGANA_IROHA, accumulator)
        }
        ListStyleType::Katakana => {
            push_alphabetic_representation(value, &KATAKANA, accumulator)
        }
        ListStyleType::KatakanaIroha => {
            push_alphabetic_representation(value, &KATAKANA_IROHA, accumulator)
        }
    }
}

/// Returns the static character that represents the value rendered using the given list-style, if
/// possible.
pub fn static_representation(list_style_type: ListStyleType) -> char {
    match list_style_type {
        ListStyleType::Disc => '•',
        ListStyleType::Circle => '◦',
        ListStyleType::Square => '▪',
        ListStyleType::DisclosureOpen => '▾',
        ListStyleType::DisclosureClosed => '‣',
        _ => panic!("No static representation for this list-style-type!"),
    }
}

/// Pushes the string that represents the value rendered using the given *alphabetic system* onto
/// the accumulator per CSS-COUNTER-STYLES § 3.1.4.
fn push_alphabetic_representation(value: i32, system: &[char], accumulator: &mut String) {
    let mut abs_value = handle_negative_value(value, accumulator);

    let mut string: SmallVec<[char; 8]> = SmallVec::new();
    while abs_value != 0 {
        // Step 1.
        abs_value = abs_value - 1;
        // Step 2.
        string.push(system[abs_value % system.len()]);
        // Step 3.
        abs_value = abs_value / system.len();
    }

    accumulator.extend(string.iter().cloned().rev())
}

/// Pushes the string that represents the value rendered using the given *numeric system* onto the
/// accumulator per CSS-COUNTER-STYLES § 3.1.5.
fn push_numeric_representation(value: i32, system: &[char], accumulator: &mut String) {
    let mut abs_value = handle_negative_value(value, accumulator);

    // Step 1.
    if abs_value == 0 {
        accumulator.push(system[0]);
        return
    }

    // Step 2.
    let mut string: SmallVec<[char; 8]> = SmallVec::new();
    while abs_value != 0 {
        // Step 2.1.
        string.push(system[abs_value % system.len()]);
        // Step 2.2.
        abs_value = abs_value / system.len();
    }

    // Step 3.
    accumulator.extend(string.iter().cloned().rev())
}

/// If the system uses a negative sign, handle negative values per CSS-COUNTER-STYLES § 2.
///
/// Returns the absolute value of the counter.
fn handle_negative_value(value: i32, accumulator: &mut String) -> usize {
    // 3. If the counter value is negative and the counter style uses a negative sign, instead
    //    generate an initial representation using the absolute value of the counter value.
    if value < 0 {
        // TODO: Support different negative signs using the 'negative' descriptor.
        // https://drafts.csswg.org/date/2015-07-16/css-counter-styles/#counter-style-negative
        accumulator.push('-');
        value.abs() as usize
    } else {
        value as usize
    }
}
