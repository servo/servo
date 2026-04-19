// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Check if String.prototype.toLocaleLowerCase supports conditional mappings defined in SpecialCasings
info: |
    The result must be derived according to the locale-insensitive case mappings in the Unicode Character
    Database (this explicitly includes not only the UnicodeData.txt file, but also all locale-insensitive
    mappings in the SpecialCasings.txt file that accompanies it).
es5id: 15.5.4.17
es6id: 21.1.3.20
---*/

// SpecialCasing.txt, conditional, language-insensitive mappings.

// <code>; <lower>; <title>; <upper>; (<condition_list>;)? # <comment>
// 03A3; 03C2; 03A3; 03A3; Final_Sigma; # GREEK CAPITAL LETTER SIGMA
// 03A3; 03C3; 03A3; 03A3; # GREEK CAPITAL LETTER SIGMA

// Final_Sigma is defined in Unicode 5.1, 3.13 Default Case Algorithms.

assert.sameValue(
  "\u03A3".toLocaleLowerCase(),
  "\u03C3",
  "Single GREEK CAPITAL LETTER SIGMA"
);

// Sigma preceded by Cased and zero or more Case_Ignorable.
assert.sameValue(
  "A\u03A3".toLocaleLowerCase(),
  "a\u03C2",
  "Sigma preceded by LATIN CAPITAL LETTER A"
);
assert.sameValue(
  "\uD835\uDCA2\u03A3".toLocaleLowerCase(),
  "\uD835\uDCA2\u03C2",
  "Sigma preceded by MATHEMATICAL SCRIPT CAPITAL G (D835 DCA2 = 1D4A2)"
);
assert.sameValue(
  "A.\u03A3".toLocaleLowerCase(),
  "a.\u03C2",
  "Sigma preceded by FULL STOP"
);
assert.sameValue(
  "A\u00AD\u03A3".toLocaleLowerCase(),
  "a\u00AD\u03C2",
  "Sigma preceded by SOFT HYPHEN (00AD)"
);
assert.sameValue(
  "A\uD834\uDE42\u03A3".toLocaleLowerCase(),
  "a\uD834\uDE42\u03C2",
  "Sigma preceded by COMBINING GREEK MUSICAL TRISEME (D834 DE42 = 1D242)"
);
assert.sameValue(
  "\u0345\u03A3".toLocaleLowerCase(),
  "\u0345\u03C3",
  "Sigma preceded by COMBINING GREEK YPOGEGRAMMENI (0345)"
);
assert.sameValue(
  "\u0391\u0345\u03A3".toLocaleLowerCase(),
  "\u03B1\u0345\u03C2",
  "Sigma preceded by GREEK CAPITAL LETTER ALPHA (0391), COMBINING GREEK YPOGEGRAMMENI (0345)"
);

// Sigma not followed by zero or more Case_Ignorable and then Cased.
assert.sameValue(
  "A\u03A3B".toLocaleLowerCase(),
  "a\u03C3b",
  "Sigma followed by LATIN CAPITAL LETTER B"
);
assert.sameValue(
  "A\u03A3\uD835\uDCA2".toLocaleLowerCase(),
  "a\u03C3\uD835\uDCA2",
  "Sigma followed by MATHEMATICAL SCRIPT CAPITAL G (D835 DCA2 = 1D4A2)"
);
assert.sameValue(
  "A\u03A3.b".toLocaleLowerCase(),
  "a\u03C3.b",
  "Sigma followed by FULL STOP"
);
assert.sameValue(
  "A\u03A3\u00ADB".toLocaleLowerCase(),
  "a\u03C3\u00ADb",
  "Sigma followed by SOFT HYPHEN (00AD)"
);
assert.sameValue(
  "A\u03A3\uD834\uDE42B".toLocaleLowerCase(),
  "a\u03C3\uD834\uDE42b",
  "Sigma followed by COMBINING GREEK MUSICAL TRISEME (D834 DE42 = 1D242)"
);
assert.sameValue(
  "A\u03A3\u0345".toLocaleLowerCase(),
  "a\u03C2\u0345",
  "Sigma followed by COMBINING GREEK YPOGEGRAMMENI (0345)"
);
assert.sameValue(
  "A\u03A3\u0345\u0391".toLocaleLowerCase(),
  "a\u03C3\u0345\u03B1",
  "Sigma followed by COMBINING GREEK YPOGEGRAMMENI (0345), GREEK CAPITAL LETTER ALPHA (0391)"
);
