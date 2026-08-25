// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Check if String.prototype.toLocaleLowerCase supports language-sensitive mappings defined in SpecialCasings (Lithuanian)
info: |
    The result must be derived according to the case mappings in the Unicode character database (this explicitly
    includes not only the UnicodeData.txt file, but also the SpecialCasings.txt file that accompanies it).
es5id: 15.5.4.16
es6id: 21.1.3.20
---*/

// SpecialCasing.txt, conditional, language-sensitive mappings (Lithuanian).

// COMBINING DOT ABOVE added when lowercasing capital I, J and I WITH OGONEK (U+012E).
// Character directly followed by character of combining class 230 (Above).
// - COMBINING GRAVE ACCENT (U+0300), combining class 230 (Above)
assert.sameValue(
  "I\u0300".toLocaleLowerCase("lt"),
  "i\u0307\u0300",
  "LATIN CAPITAL LETTER I followed by COMBINING GRAVE ACCENT"
);
assert.sameValue(
  "J\u0300".toLocaleLowerCase("lt"),
  "j\u0307\u0300",
  "LATIN CAPITAL LETTER J followed by COMBINING GRAVE ACCENT"
);
assert.sameValue(
  "\u012E\u0300".toLocaleLowerCase("lt"),
  "\u012F\u0307\u0300",
  "LATIN CAPITAL LETTER I WITH OGONEK followed by COMBINING GRAVE ACCENT"
);


// COMBINING DOT ABOVE added when lowercasing capital I, J and I WITH OGONEK (U+012E).
// Character directly followed by character of combining class 230 (Above).
// - MUSICAL SYMBOL COMBINING DOIT (U+1D185, D834 DD85), combining class 230 (Above)
assert.sameValue(
  "I\uD834\uDD85".toLocaleLowerCase("lt"),
  "i\u0307\uD834\uDD85",
  "LATIN CAPITAL LETTER I followed by MUSICAL SYMBOL COMBINING DOIT"
);
assert.sameValue(
  "J\uD834\uDD85".toLocaleLowerCase("lt"),
  "j\u0307\uD834\uDD85",
  "LATIN CAPITAL LETTER J followed by MUSICAL SYMBOL COMBINING DOIT"
);
assert.sameValue(
  "\u012E\uD834\uDD85".toLocaleLowerCase("lt"),
  "\u012F\u0307\uD834\uDD85",
  "LATIN CAPITAL LETTER I WITH OGONEK followed by MUSICAL SYMBOL COMBINING DOIT"
);


// COMBINING DOT ABOVE added when lowercasing capital I, J and I WITH OGONEK (U+012E).
// Character not directly followed by character of combining class 230 (Above).
// - COMBINING RING BELOW (U+0325), combining class 220 (Below)
// - COMBINING GRAVE ACCENT (U+0300), combining class 230 (Above)
assert.sameValue(
  "I\u0325\u0300".toLocaleLowerCase("lt"),
  "i\u0307\u0325\u0300",
  "LATIN CAPITAL LETTER I followed by COMBINING RING BELOW, COMBINING GRAVE ACCENT"
);
assert.sameValue(
  "J\u0325\u0300".toLocaleLowerCase("lt"),
  "j\u0307\u0325\u0300",
  "LATIN CAPITAL LETTER J followed by COMBINING RING BELOW, COMBINING GRAVE ACCENT"
);
assert.sameValue(
  "\u012E\u0325\u0300".toLocaleLowerCase("lt"),
  "\u012F\u0307\u0325\u0300",
  "LATIN CAPITAL LETTER I WITH OGONEK followed by COMBINING RING BELOW, COMBINING GRAVE ACCENT"
);


// COMBINING DOT ABOVE added when lowercasing capital I, J and I WITH OGONEK (U+012E).
// Character not directly followed by character of combining class 230 (Above).
// - PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE (U+101FD, D800 DDFD), combining class 220 (Below)
// - COMBINING GRAVE ACCENT (U+0300), combining class 230 (Above)
assert.sameValue(
  "I\uD800\uDDFD\u0300".toLocaleLowerCase("lt"),
  "i\u0307\uD800\uDDFD\u0300",
  "LATIN CAPITAL LETTER I followed by PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE, COMBINING GRAVE ACCENT"
);
assert.sameValue(
  "J\uD800\uDDFD\u0300".toLocaleLowerCase("lt"),
  "j\u0307\uD800\uDDFD\u0300",
  "LATIN CAPITAL LETTER J followed by PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE, COMBINING GRAVE ACCENT"
);
assert.sameValue(
  "\u012E\uD800\uDDFD\u0300".toLocaleLowerCase("lt"),
  "\u012F\u0307\uD800\uDDFD\u0300",
  "LATIN CAPITAL LETTER I WITH OGONEK followed by PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE, COMBINING GRAVE ACCENT"
);


// COMBINING DOT ABOVE added when lowercasing capital I, J and I WITH OGONEK (U+012E).
// Character not directly followed by character of combining class 230 (Above).
// - COMBINING RING BELOW (U+0325), combining class 220 (Below)
// - MUSICAL SYMBOL COMBINING DOIT (U+1D185, D834 DD85), combining class 230 (Above)
assert.sameValue(
  "I\u0325\uD834\uDD85".toLocaleLowerCase("lt"),
  "i\u0307\u0325\uD834\uDD85",
  "LATIN CAPITAL LETTER I followed by COMBINING RING BELOW, MUSICAL SYMBOL COMBINING DOIT"
);
assert.sameValue(
  "J\u0325\uD834\uDD85".toLocaleLowerCase("lt"),
  "j\u0307\u0325\uD834\uDD85",
  "LATIN CAPITAL LETTER J followed by COMBINING RING BELOW, MUSICAL SYMBOL COMBINING DOIT"
);
assert.sameValue(
  "\u012E\u0325\uD834\uDD85".toLocaleLowerCase("lt"),
  "\u012F\u0307\u0325\uD834\uDD85",
  "LATIN CAPITAL LETTER I WITH OGONEK followed by COMBINING RING BELOW, MUSICAL SYMBOL COMBINING DOIT"
);


// COMBINING DOT ABOVE added when lowercasing capital I, J and I WITH OGONEK (U+012E).
// Character not directly followed by character of combining class 230 (Above).
// - PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE (U+101FD, D800 DDFD), combining class 220 (Below)
// - MUSICAL SYMBOL COMBINING DOIT (U+1D185, D834 DD85), combining class 230 (Above)
assert.sameValue(
  "I\uD800\uDDFD\uD834\uDD85".toLocaleLowerCase("lt"),
  "i\u0307\uD800\uDDFD\uD834\uDD85",
  "LATIN CAPITAL LETTER I followed by PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE, MUSICAL SYMBOL COMBINING DOIT"
);
assert.sameValue(
  "J\uD800\uDDFD\uD834\uDD85".toLocaleLowerCase("lt"),
  "j\u0307\uD800\uDDFD\uD834\uDD85",
  "LATIN CAPITAL LETTER J followed by PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE, MUSICAL SYMBOL COMBINING DOIT"
);
assert.sameValue(
  "\u012E\uD800\uDDFD\uD834\uDD85".toLocaleLowerCase("lt"),
  "\u012F\u0307\uD800\uDDFD\uD834\uDD85",
  "LATIN CAPITAL LETTER I WITH OGONEK followed by PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE, MUSICAL SYMBOL COMBINING DOIT"
);


// COMBINING DOT ABOVE not added when character is followed by a character of combining class 0.
// - COMBINING GRAVE ACCENT (U+0300), combining class 230 (Above)
assert.sameValue(
  "IA\u0300".toLocaleLowerCase("lt"),
  "ia\u0300",
  "LATIN CAPITAL LETTER I followed by LATIN CAPITAL LETTER A, COMBINING GRAVE ACCENT"
);
assert.sameValue(
  "JA\u0300".toLocaleLowerCase("lt"),
  "ja\u0300",
  "LATIN CAPITAL LETTER J followed by LATIN CAPITAL LETTER A, COMBINING GRAVE ACCENT"
);
assert.sameValue(
  "\u012EA\u0300".toLocaleLowerCase("lt"),
  "\u012Fa\u0300",
  "LATIN CAPITAL LETTER I WITH OGONEK followed by LATIN CAPITAL LETTER A, COMBINING GRAVE ACCENT"
);


// COMBINING DOT ABOVE not added when character is followed by a character of combining class 0.
// - MUSICAL SYMBOL COMBINING DOIT (U+1D185, D834 DD85), combining class 230 (Above)
assert.sameValue(
  "IA\uD834\uDD85".toLocaleLowerCase("lt"),
  "ia\uD834\uDD85",
  "LATIN CAPITAL LETTER I followed by LATIN CAPITAL LETTER A, MUSICAL SYMBOL COMBINING DOIT"
);
assert.sameValue(
  "JA\uD834\uDD85".toLocaleLowerCase("lt"),
  "ja\uD834\uDD85",
  "LATIN CAPITAL LETTER J followed by LATIN CAPITAL LETTER A, MUSICAL SYMBOL COMBINING DOIT"
);
assert.sameValue(
  "\u012EA\uD834\uDD85".toLocaleLowerCase("lt"),
  "\u012Fa\uD834\uDD85",
  "LATIN CAPITAL LETTER I WITH OGONEK (U+012E) followed by LATIN CAPITAL LETTER A, MUSICAL SYMBOL COMBINING DOIT"
);


// Precomposed characters with accents above.
assert.sameValue(
  "\u00CC".toLocaleLowerCase("lt"),
  "\u0069\u0307\u0300",
  "LATIN CAPITAL LETTER I WITH GRAVE"
);
assert.sameValue(
  "\u00CD".toLocaleLowerCase("lt"),
  "\u0069\u0307\u0301",
  "LATIN CAPITAL LETTER I WITH ACUTE"
);
assert.sameValue(
  "\u0128".toLocaleLowerCase("lt"),
  "\u0069\u0307\u0303",
  "LATIN CAPITAL LETTER I WITH TILDE"
);
