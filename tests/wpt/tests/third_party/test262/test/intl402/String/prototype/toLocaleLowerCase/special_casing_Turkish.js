// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Check if String.prototype.toLocaleLowerCase supports language-sensitive mappings defined in SpecialCasings (Turkish)
info: |
    The result must be derived according to the case mappings in the Unicode character database (this explicitly
    includes not only the UnicodeData.txt file, but also the SpecialCasings.txt file that accompanies it).
es5id: 15.5.4.16
es6id: 21.1.3.20
---*/

// SpecialCasing.txt, conditional, language-sensitive mappings (Turkish).

// LATIN CAPITAL LETTER I WITH DOT ABOVE (U+0130) changed to LATIN SMALL LETTER I when lowercasing.
assert.sameValue(
  "\u0130".toLocaleLowerCase("tr"),
  "i",
  "LATIN CAPITAL LETTER I WITH DOT ABOVE"
);


// COMBINING DOT ABOVE (U+0307) removed after LATIN CAPITAL LETTER I when lowercasing.
// - COMBINING DOT BELOW (U+0323), combining class 220 (Below)
// - PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE (U+101FD = D800 DDFD), combining class 220 (Below)
assert.sameValue(
  "I\u0307".toLocaleLowerCase("tr"),
  "i",
  "LATIN CAPITAL LETTER I followed by COMBINING DOT ABOVE"
);
assert.sameValue(
  "I\u0323\u0307".toLocaleLowerCase("tr"),
  "i\u0323",
  "LATIN CAPITAL LETTER I followed by COMBINING DOT BELOW, COMBINING DOT ABOVE"
);
assert.sameValue(
  "I\uD800\uDDFD\u0307".toLocaleLowerCase("tr"),
  "i\uD800\uDDFD",
  "LATIN CAPITAL LETTER I followed by PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE, COMBINING DOT ABOVE"
);


// COMBINING DOT ABOVE (U+0307) not removed when character is preceded by a character of combining class 0.
assert.sameValue(
  "IA\u0307".toLocaleLowerCase("tr"),
  "\u0131a\u0307",
  "LATIN CAPITAL LETTER I followed by LATIN CAPITAL LETTER A, COMBINING DOT ABOVE"
);


// COMBINING DOT ABOVE (U+0307) not removed when character is preceded by a character of combining class 230.
// - COMBINING GRAVE ACCENT (U+0300), combining class 230 (Above)
// - MUSICAL SYMBOL COMBINING DOIT (U+1D185, D834 DD85), combining class 230 (Above)
assert.sameValue(
  "I\u0300\u0307".toLocaleLowerCase("tr"),
  "\u0131\u0300\u0307",
  "LATIN CAPITAL LETTER I followed by COMBINING GRAVE ACCENT, COMBINING DOT ABOVE"
);
assert.sameValue(
  "I\uD834\uDD85\u0307".toLocaleLowerCase("tr"),
  "\u0131\uD834\uDD85\u0307",
  "LATIN CAPITAL LETTER I followed by MUSICAL SYMBOL COMBINING DOIT, COMBINING DOT ABOVE"
);


// LATIN CAPITAL LETTER I changed to LATIN SMALL LETTER DOTLESS I (U+0131) when lowercasing.
assert.sameValue(
  "I".toLocaleLowerCase("tr"),
  "\u0131",
  "LATIN CAPITAL LETTER I"
);


// No changes when lowercasing LATIN SMALL LETTER I and LATIN SMALL LETTER DOTLESS I (U+0131).
assert.sameValue(
  "i".toLocaleLowerCase("tr"),
  "i",
  "LATIN SMALL LETTER I"
);
assert.sameValue(
  "\u0131".toLocaleLowerCase("tr"),
  "\u0131",
  "LATIN SMALL LETTER DOTLESS I"
);
