// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  ignoreCase match should perform Canonicalize both on input and pattern.
info: bugzilla.mozilla.org/show_bug.cgi?id=1280046
esid: pending
---*/

// Each element [code1, upper, code2] satisfies the following condition:
//   ToUpperCase(code1) == upper
//   ToUpperCase(code2) == upper
var pairs =
    [
        // U+00B5: MICRO SIGN
        // U+039C: GREEK CAPITAL LETTER MU
        // U+03BC: GREEK SMALL LETTER MU
        ["\u00B5", "\u039C", "\u03BC"],
        // U+0345: COMBINING GREEK YPOGEGRAMMENI
        // U+0399: GREEK CAPITAL LETTER IOTA
        // U+03B9: GREEK SMALL LETTER IOTA
        ["\u0345", "\u0399", "\u03B9"],
        // U+03C2: GREEK SMALL LETTER FINAL SIGMA
        // U+03A3: GREEK CAPITAL LETTER SIGMA
        // U+03C3: GREEK SMALL LETTER SIGMA
        ["\u03C2", "\u03A3", "\u03C3"],
        // U+03D0: GREEK BETA SYMBOL
        // U+0392: GREEK CAPITAL LETTER BETA
        // U+03B2: GREEK SMALL LETTER BETA
        ["\u03D0", "\u0392", "\u03B2"],
        // U+03D1: GREEK THETA SYMBOL
        // U+0398: GREEK CAPITAL LETTER THETA
        // U+03B8: GREEK SMALL LETTER THETA
        ["\u03D1", "\u0398", "\u03B8"],
        // U+03D5: GREEK PHI SYMBOL
        // U+03A6: GREEK CAPITAL LETTER PHI
        // U+03C6: GREEK SMALL LETTER PHI
        ["\u03D5", "\u03A6", "\u03C6"],
        // U+03D6: GREEK PI SYMBOL
        // U+03A0: GREEK CAPITAL LETTER PI
        // U+03C0: GREEK SMALL LETTER PI
        ["\u03D6", "\u03A0", "\u03C0"],
        // U+03F0: GREEK KAPPA SYMBOL
        // U+039A: GREEK CAPITAL LETTER KAPPA
        // U+03BA: GREEK SMALL LETTER KAPPA
        ["\u03F0", "\u039A", "\u03BA"],
        // U+03F1: GREEK RHO SYMBOL
        // U+03A1: GREEK CAPITAL LETTER RHO
        // U+03C1: GREEK SMALL LETTER RHO
        ["\u03F1", "\u03A1", "\u03C1"],
        // U+03F5: GREEK LUNATE EPSILON SYMBOL
        // U+0395: GREEK CAPITAL LETTER EPSILON
        // U+03B5: GREEK SMALL LETTER EPSILON
        ["\u03F5", "\u0395", "\u03B5"],
        // U+1E9B: LATIN SMALL LETTER LONG S WITH DOT ABOVE
        // U+1E60: LATIN CAPITAL LETTER S WITH DOT ABOVE
        // U+1E61: LATIN SMALL LETTER S WITH DOT ABOVE
        ["\u1E9B", "\u1E60", "\u1E61"],
        // U+1FBE: GREEK PROSGEGRAMMENI
        // U+0399: GREEK CAPITAL LETTER IOTA
        // U+03B9: GREEK SMALL LETTER IOTA
        ["\u1FBE", "\u0399", "\u03B9"],
    ];

for (var [code1, upper, code2] of pairs) {
    assert.sameValue(new RegExp(code1, "i").test(code2), true);
    assert.sameValue(new RegExp(code1, "i").test(upper), true);
    assert.sameValue(new RegExp(upper, "i").test(code1), true);
    assert.sameValue(new RegExp(upper, "i").test(code2), true);
    assert.sameValue(new RegExp(code2, "i").test(code1), true);
    assert.sameValue(new RegExp(code2, "i").test(upper), true);
}
