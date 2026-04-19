// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.tolowercase
description: >
    Check if String.prototype.toLowerCase supports conditional mappings defined in SpecialCasings,
    test Final_Sigma context with Mongolian Vowel Separator
info: |
    The result must be derived according to the locale-insensitive case mappings in the Unicode Character
    Database (this explicitly includes not only the UnicodeData.txt file, but also all locale-insensitive
    mappings in the SpecialCasings.txt file that accompanies it).
features: [u180e]
---*/

// SpecialCasing.txt, conditional, language-insensitive mappings.

// <code>; <lower>; <title>; <upper>; (<condition_list>;)? # <comment>
// 03A3; 03C2; 03A3; 03A3; Final_Sigma; # GREEK CAPITAL LETTER SIGMA
// 03A3; 03C3; 03A3; 03A3; # GREEK CAPITAL LETTER SIGMA

// Final_Sigma is defined in Unicode 8.0, 3.13 Default Case Algorithms
// General_Category of Mongolian Vowel Separator is Cf (Format), characters in Cf are Case_Ignorable.


// Sigma preceded by Mongolian Vowel Separator.
assert.sameValue(
  "A\u180E\u03A3".toLowerCase(),
  "a\u180E\u03C2",
  "Sigma preceded by LATIN CAPITAL LETTER A, MONGOLIAN VOWEL SEPARATOR"
);
assert.sameValue(
  "A\u180E\u03A3B".toLowerCase(),
  "a\u180E\u03C3b",
  "Sigma preceded by LATIN CAPITAL LETTER A, MONGOLIAN VOWEL SEPARATOR, followed by LATIN CAPITAL LETTER B"
);

// Sigma followed by Mongolian Vowel Separator.
assert.sameValue(
  "A\u03A3\u180E".toLowerCase(),
  "a\u03C2\u180E",
  "Sigma preceded by LATIN CAPITAL LETTER A, followed by MONGOLIAN VOWEL SEPARATOR"
);
assert.sameValue(
  "A\u03A3\u180EB".toLowerCase(),
  "a\u03C3\u180Eb",
  "Sigma preceded by LATIN CAPITAL LETTER A, followed by MONGOLIAN VOWEL SEPARATOR, LATIN CAPITAL LETTER B"
);

// Sigma preceded and followed by Mongolian Vowel Separator.
assert.sameValue(
  "A\u180E\u03A3\u180E".toLowerCase(),
  "a\u180E\u03C2\u180E",
  "Sigma preceded by LATIN CAPITAL LETTER A, MONGOLIAN VOWEL SEPARATOR, followed by MONGOLIAN VOWEL SEPARATOR"
);
assert.sameValue(
  "A\u180E\u03A3\u180EB".toLowerCase(),
  "a\u180E\u03C3\u180Eb",
  "Sigma preceded by LATIN CAPITAL LETTER A, MONGOLIAN VOWEL SEPARATOR, followed by MONGOLIAN VOWEL SEPARATOR, LATIN CAPITAL LETTER B"
);
