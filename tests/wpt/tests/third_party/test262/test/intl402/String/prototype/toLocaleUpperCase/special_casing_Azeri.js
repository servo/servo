// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Check if String.prototype.toLocaleUpperCase supports language-sensitive mappings defined in SpecialCasings (Azeri)
info: |
    The result must be derived according to the case mappings in the Unicode character database (this explicitly
    includes not only the UnicodeData.txt file, but also the SpecialCasings.txt file that accompanies it).
es5id: 15.5.4.16
es6id: 21.1.3.21
---*/

// SpecialCasing.txt, conditional, language-sensitive mappings (Azeri).

// LATIN CAPITAL LETTER I WITH DOT ABOVE (U+0130) not changed when uppercasing.
assert.sameValue(
  "\u0130".toLocaleUpperCase("az"),
  "\u0130",
  "LATIN CAPITAL LETTER I WITH DOT ABOVE"
);


// LATIN CAPITAL LETTER I not changed when uppercasing.
assert.sameValue(
  "I".toLocaleUpperCase("az"),
  "I",
  "LATIN CAPITAL LETTER I"
);


// LATIN SMALL LETTER I changed to LATIN CAPITAL LETTER I WITH DOT ABOVE (U+0130) when uppercasing.
assert.sameValue(
  "i".toLocaleUpperCase("az"),
  "\u0130",
  "LATIN SMALL LETTER I"
);


// LATIN SMALL LETTER DOTLESS I (U+0131) changed to LATIN CAPITAL LETTER I when uppercasing.
assert.sameValue(
  "\u0131".toLocaleUpperCase("az"),
  "I",
  "LATIN SMALL LETTER DOTLESS I"
);
