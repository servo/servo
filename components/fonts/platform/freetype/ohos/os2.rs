/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use servo_base::text::UnicodeBlock;

/// maps a unicode block to the corresponding unicode range in os/2 table.
/// <https://learn.microsoft.com/en-us/typography/opentype/spec/os2#ur>
pub fn unicode_block_to_os2_bits(block: Option<UnicodeBlock>) -> Option<u128> {
    match block? {
        UnicodeBlock::BasicLatin => Some(1_u128),
        UnicodeBlock::Latin1Supplement => Some(1_u128 << 1),
        UnicodeBlock::LatinExtendedA => Some(1_u128 << 2),
        UnicodeBlock::LatinExtendedB => Some(1_u128 << 3),
        UnicodeBlock::IPAExtensions |
        UnicodeBlock::PhoneticExtensions |
        UnicodeBlock::PhoneticExtensionsSupplement => Some(1_u128 << 4),
        UnicodeBlock::SpacingModifierLetters | UnicodeBlock::ModifierToneLetters => {
            Some(1_u128 << 5)
        },
        UnicodeBlock::CombiningDiacriticalMarks |
        UnicodeBlock::CombiningDiacriticalMarksSupplement => Some(1_u128 << 6),
        UnicodeBlock::GreekandCoptic => Some(1_u128 << 7),
        UnicodeBlock::Coptic => Some(1_u128 << 8),
        UnicodeBlock::Cyrillic |
        UnicodeBlock::CyrillicSupplement |
        UnicodeBlock::CyrillicExtendedA |
        UnicodeBlock::CyrillicExtendedB => Some(1_u128 << 9),
        UnicodeBlock::Armenian => Some(1_u128 << 10),

        UnicodeBlock::Hebrew => Some(1_u128 << 11),
        UnicodeBlock::Vai => Some(1_u128 << 12),
        UnicodeBlock::Arabic | UnicodeBlock::ArabicSupplement => Some(1_u128 << 13),
        UnicodeBlock::NKo => Some(1_u128 << 14),
        UnicodeBlock::Devanagari => Some(1_u128 << 15),
        UnicodeBlock::Gurmukhi => Some(1_u128 << 17),
        UnicodeBlock::Gujarati => Some(1_u128 << 18),
        UnicodeBlock::Tamil => Some(1_u128 << 20),

        UnicodeBlock::Telugu => Some(1_u128 << 21),
        UnicodeBlock::Kannada => Some(1_u128 << 22),
        UnicodeBlock::Malayalam => Some(1_u128 << 23),
        UnicodeBlock::Thai => Some(1_u128 << 24),
        UnicodeBlock::Lao => Some(1_u128 << 25),
        UnicodeBlock::Georgian | UnicodeBlock::GeorgianSupplement => Some(1_u128 << 26),
        UnicodeBlock::Balinese => Some(1_u128 << 27),
        UnicodeBlock::HangulJamo => Some(1_u128 << 28),
        UnicodeBlock::LatinExtendedAdditional |
        UnicodeBlock::LatinExtendedC |
        UnicodeBlock::LatinExtendedD => Some(1_u128 << 29),
        UnicodeBlock::GreekExtended => Some(1_u128 << 30),

        UnicodeBlock::GeneralPunctuation | UnicodeBlock::SupplementalPunctuation => {
            Some(1_u128 << 31)
        },
        UnicodeBlock::SuperscriptsandSubscripts => Some(1_u128 << 32),
        UnicodeBlock::CurrencySymbols => Some(1_u128 << 33),
        UnicodeBlock::CombiningDiacriticalMarksforSymbols => Some(1_u128 << 34),
        UnicodeBlock::LetterlikeSymbols => Some(1_u128 << 35),
        UnicodeBlock::NumberForms => Some(1_u128 << 36),
        UnicodeBlock::Arrows |
        UnicodeBlock::SupplementalArrowsA |
        UnicodeBlock::SupplementalArrowsB |
        UnicodeBlock::MiscellaneousSymbolsandArrows => Some(1_u128 << 37),
        UnicodeBlock::MathematicalOperators |
        UnicodeBlock::SupplementalMathematicalOperators |
        UnicodeBlock::MiscellaneousMathematicalSymbolsA |
        UnicodeBlock::MiscellaneousMathematicalSymbolsB => Some(1_u128 << 38),
        UnicodeBlock::MiscellaneousTechnical => Some(1_u128 << 39),
        UnicodeBlock::ControlPictures => Some(1_u128 << 40),

        UnicodeBlock::OpticalCharacterRecognition => Some(1_u128 << 41),
        UnicodeBlock::EnclosedAlphanumerics => Some(1_u128 << 42),
        UnicodeBlock::BoxDrawing => Some(1_u128 << 43),
        UnicodeBlock::BlockElements => Some(1_u128 << 44),
        UnicodeBlock::GeometricShapes => Some(1_u128 << 45),
        UnicodeBlock::MiscellaneousSymbols => Some(1_u128 << 46),
        UnicodeBlock::Dingbats => Some(1_u128 << 47),
        UnicodeBlock::CJKSymbolsandPunctuation => Some(1_u128 << 48),
        UnicodeBlock::Hiragana => Some(1_u128 << 49),
        UnicodeBlock::Katakana | UnicodeBlock::KatakanaPhoneticExtensions => Some(1_u128 << 50),

        UnicodeBlock::Bopomofo | UnicodeBlock::BopomofoExtended => Some(1_u128 << 51),
        UnicodeBlock::HangulCompatibilityJamo => Some(1_u128 << 52),
        UnicodeBlock::Phagspa => Some(1_u128 << 53),
        UnicodeBlock::EnclosedCJKLettersandMonths => Some(1_u128 << 54),
        UnicodeBlock::CJKCompatibility => Some(1_u128 << 55),
        UnicodeBlock::HangulSyllables => Some(1_u128 << 56),
        UnicodeBlock::Phoenician => Some(1_u128 << 58),
        UnicodeBlock::CJKUnifiedIdeographs |
        UnicodeBlock::CJKRadicalsSupplement |
        UnicodeBlock::KangxiRadicals |
        UnicodeBlock::IdeographicDescriptionCharacters |
        UnicodeBlock::CJKUnifiedIdeographsExtensionA |
        UnicodeBlock::CJKUnifiedIdeographsExtensionB |
        UnicodeBlock::Kanbun => Some(1_u128 << 59),
        UnicodeBlock::PrivateUseArea => Some(1_u128 << 60),

        UnicodeBlock::CJKStrokes |
        UnicodeBlock::CJKCompatibilityIdeographs |
        UnicodeBlock::CJKCompatibilityIdeographsSupplement => Some(1_u128 << 61),
        UnicodeBlock::AlphabeticPresentationForms => Some(1_u128 << 62),
        UnicodeBlock::ArabicPresentationFormsA => Some(1_u128 << 63),
        UnicodeBlock::CombiningHalfMarks => Some(1_u128 << 64),
        UnicodeBlock::VerticalForms | UnicodeBlock::CJKCompatibilityForms => Some(1_u128 << 65),
        UnicodeBlock::SmallFormVariants => Some(1_u128 << 66),
        UnicodeBlock::ArabicPresentationFormsB => Some(1_u128 << 67),
        UnicodeBlock::HalfwidthandFullwidthForms => Some(1_u128 << 68),
        UnicodeBlock::Specials => Some(1_u128 << 69),
        UnicodeBlock::Tibetan => Some(1_u128 << 70),

        UnicodeBlock::Syriac => Some(1_u128 << 71),
        UnicodeBlock::Thaana => Some(1_u128 << 72),
        UnicodeBlock::Sinhala => Some(1_u128 << 73),
        UnicodeBlock::Myanmar => Some(1_u128 << 74),
        UnicodeBlock::Ethiopic |
        UnicodeBlock::EthiopicSupplement |
        UnicodeBlock::EthiopicExtended => Some(1_u128 << 75),
        UnicodeBlock::Cherokee => Some(1_u128 << 76),
        UnicodeBlock::UnifiedCanadianAboriginalSyllabics => Some(1_u128 << 77),
        UnicodeBlock::Ogham => Some(1_u128 << 78),
        UnicodeBlock::Runic => Some(1_u128 << 79),
        UnicodeBlock::Khmer | UnicodeBlock::KhmerSymbols => Some(1_u128 << 80),

        UnicodeBlock::Mongolian => Some(1_u128 << 81),
        UnicodeBlock::BraillePatterns => Some(1_u128 << 82),
        UnicodeBlock::YiSyllables | UnicodeBlock::YiRadicals => Some(1_u128 << 83),
        UnicodeBlock::Tagalog |
        UnicodeBlock::Hanunoo |
        UnicodeBlock::Buhid |
        UnicodeBlock::Tagbanwa => Some(1_u128 << 84),
        UnicodeBlock::OldItalic => Some(1_u128 << 85),
        UnicodeBlock::Gothic => Some(1_u128 << 86),
        UnicodeBlock::Deseret => Some(1_u128 << 87),
        UnicodeBlock::ByzantineMusicalSymbols |
        UnicodeBlock::MusicalSymbols |
        UnicodeBlock::AncientGreekMusicalNotation => Some(1_u128 << 88),
        UnicodeBlock::MathematicalAlphanumericSymbols => Some(1_u128 << 89),

        UnicodeBlock::VariationSelectors | UnicodeBlock::VariationSelectorsSupplement => {
            Some(1_u128 << 91)
        },
        UnicodeBlock::Tags => Some(1_u128 << 92),
        UnicodeBlock::Limbu => Some(1_u128 << 93),
        UnicodeBlock::TaiLe => Some(1_u128 << 94),
        UnicodeBlock::NewTaiLue => Some(1_u128 << 95),
        UnicodeBlock::Buginese => Some(1_u128 << 96),
        UnicodeBlock::Glagolitic => Some(1_u128 << 97),
        UnicodeBlock::Tifinagh => Some(1_u128 << 98),
        UnicodeBlock::YijingHexagramSymbols => Some(1_u128 << 99),
        UnicodeBlock::SylotiNagri => Some(1_u128 << 100),

        UnicodeBlock::LinearBSyllabary |
        UnicodeBlock::LinearBIdeograms |
        UnicodeBlock::AegeanNumbers => Some(1_u128 << 101),
        UnicodeBlock::AncientGreekNumbers => Some(1_u128 << 102),
        UnicodeBlock::Ugaritic => Some(1_u128 << 103),
        UnicodeBlock::OldPersian => Some(1_u128 << 104),
        UnicodeBlock::Shavian => Some(1_u128 << 105),
        UnicodeBlock::Osmanya => Some(1_u128 << 106),
        UnicodeBlock::CypriotSyllabary => Some(1_u128 << 107),
        UnicodeBlock::Kharoshthi => Some(1_u128 << 108),
        UnicodeBlock::TaiXuanJingSymbols => Some(1_u128 << 109),
        UnicodeBlock::Cuneiform | UnicodeBlock::CuneiformNumbersandPunctuation => {
            Some(1_u128 << 110)
        },

        UnicodeBlock::CountingRodNumerals => Some(1_u128 << 111),
        UnicodeBlock::Sundanese => Some(1_u128 << 112),
        UnicodeBlock::Lepcha => Some(1_u128 << 113),
        UnicodeBlock::OlChiki => Some(1_u128 << 114),
        UnicodeBlock::Saurashtra => Some(1_u128 << 115),
        UnicodeBlock::KayahLi => Some(1_u128 << 116),
        UnicodeBlock::Rejang => Some(1_u128 << 117),
        UnicodeBlock::Cham => Some(1_u128 << 118),
        UnicodeBlock::AncientSymbols => Some(1_u128 << 119),
        UnicodeBlock::PhaistosDisc => Some(1_u128 << 120),

        UnicodeBlock::Carian | UnicodeBlock::Lycian | UnicodeBlock::Lydian => Some(1_u128 << 121),
        UnicodeBlock::DominoTiles | UnicodeBlock::MahjongTiles => Some(1_u128 << 122),

        _ => None,
    }
}
