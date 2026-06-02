var baseChars = {
    "emoji": "\u{1fae8}",
    "cjk": "\u{8279}",
    "math": "\u{2205}"
};

var variationSelectors = {
    "emoji": ["\u{fe0e}", "\u{fe0f}"],
    "cjk": ["", "\u{FE00}", "\u{FE01}", "\u{e0100}", "\u{e0101}",
        "\u{e0102}"
    ],
    "math": ["", "\u{FE00}"]
};

var families = {
    "emoji": ["ColorEmojiFont", "MonoEmojiFont",
        "EmojiFontWithBaseCharOnly",
        "sans-serif"
    ],
    "cjk": ["CJKFontWithVS", "CJKFontWithBaseCharOnly",
        "sans-serif"
    ],
    "math": ["MathFontWithVS", "MathFontWithBaseCharOnly",
        "sans-serif"
    ]
};

var variationSequenceFamilies = new Map([
    ["\u{1fae8}\u{fe0e}", "MonoEmojiFont"],
    ["\u{1fae8}\u{fe0f}", "ColorEmojiFont"],
    ["\u{8279}\u{fe00}", "CJKFontWithVS"],
    ["\u{8279}\u{fe01}", "CJKFontWithVS"],
    ["\u{8279}\u{e0100}", "CJKFontWithVS"],
    ["\u{8279}\u{e0101}", "CJKFontWithVS"],
    ["\u{8279}\u{e0102}", "CJKFontWithVS"],
    ["\u{2205}\u{FE00}", "MathFontWithVS"]
]);

var baseCharFamilies = new Map([
    ["\u{1fae8}", new Set(["MonoEmojiFont", "ColorEmojiFont",
        "EmojiFontWithBaseCharOnly"
    ])],
    ["\u{8279}", new Set(["CJKFontWithVS",
        "CJKFontWithBaseCharOnly"
    ])],
    ["\u{2205}", new Set(["MathFontWithVS",
        "MathFontWithBaseCharOnly"
    ])]
]);

const range = function*(l) {
    for (let i = 0; i < l; i += 1) yield i;
}
const isEmpty = arr =>
    arr.length === 0;

const permutations =
    function*(a) {
  const r = arguments[1] || [];
  if (isEmpty(a))
    yield r;
  for (let i of range(a.length)) {
    const aa = [...a];
    const rr = [...r, ...aa.splice(i, 1)];
    yield* permutations(aa, rr);
  }
}

function getMatchedFamilyForVariationSequence(
    familyList, baseCharacter, variationSelector) {
  const variationSequence = baseCharacter + variationSelector;
  // First try to find a match for the whole variation sequence.
  if (variationSequenceFamilies.has(variationSequence)) {
    const matchedFamily = variationSequenceFamilies.get(variationSequence);
    if (familyList.includes(matchedFamily)) {
      return matchedFamily;
    }
  }
  // If failed, try to match only the base character from the
  // variation sequence.
  if (baseCharFamilies.has(baseCharacter)) {
    const eligibleFamilies = baseCharFamilies.get(baseCharacter);
    const matchedFamilies =
        familyList.filter(value => eligibleFamilies.has(value));
    if (matchedFamilies.length) {
      return matchedFamilies[0];
    }
  }
  // We should not reach here, we should always match one of the
  // specified web fonts in the tests.
  return "";
}

function generateContent(
    families, baseChar, variationSelectors, getFontFamilyValue) {
  var rootElem = document.createElement('div');
  // We want to test all possible combinations of variation
  // selectors and font-family list values. For the refs,
  // we explicitly specify the font that we expect to be
  // matched from the maps at the beginning of the files.
  const allFamiliesLists = permutations(families);
  for (const familyList of allFamiliesLists) {
    for (const variationSelector of variationSelectors) {
      const contentSpan = document.createElement("span");
      contentSpan.textContent = baseChar + variationSelector;
      contentSpan.style.fontFamily =
          getFontFamilyValue(familyList, baseChar, variationSelector);
      rootElem.appendChild(contentSpan);
    }
  }
  document.body.appendChild(rootElem);
}

function generateVariationSequenceTests(type) {
  var getFontFamilyValue = (familyList, baseChar, variationSelector) => {
    return familyList.join(', ');
  }
  generateContent(families[type], baseChars[type], variationSelectors[type], getFontFamilyValue);
}

function generateVariationSequenceRefs(type) {
  generateContent(
      families[type], baseChars[type], variationSelectors[type],
      getMatchedFamilyForVariationSequence);
}
