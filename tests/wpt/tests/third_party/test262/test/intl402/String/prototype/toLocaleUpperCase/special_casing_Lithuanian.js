// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Check if String.prototype.toLocaleUpperCase supports language-sensitive mappings defined in SpecialCasings (Lithuanian)
info: |
    The result must be derived according to the case mappings in the Unicode character database (this explicitly
    includes not only the UnicodeData.txt file, but also the SpecialCasings.txt file that accompanies it).
es5id: 15.5.4.16
es6id: 21.1.3.21
---*/

// SpecialCasing.txt, conditional, language-sensitive mappings (Lithuanian).

// COMBINING DOT ABOVE (U+0307) not removed when uppercasing capital I and J.
assert.sameValue(
  "I\u0307".toLocaleUpperCase("lt"),
  "I\u0307",
  "COMBINING DOT ABOVE preceded by LATIN CAPITAL LETTER I"
);
assert.sameValue(
  "J\u0307".toLocaleUpperCase("lt"),
  "J\u0307",
  "COMBINING DOT ABOVE preceded by LATIN CAPITAL LETTER J"
);


// Code points with Soft_Dotted property (Unicode 5.1, PropList.txt)
var softDotted = [
  "\u0069", "\u006A",   // LATIN SMALL LETTER I..LATIN SMALL LETTER J
  "\u012F",             // LATIN SMALL LETTER I WITH OGONEK
  "\u0249",             // LATIN SMALL LETTER J WITH STROKE
  "\u0268",             // LATIN SMALL LETTER I WITH STROKE
  "\u029D",             // LATIN SMALL LETTER J WITH CROSSED-TAIL
  "\u02B2",             // MODIFIER LETTER SMALL J
  "\u03F3",             // GREEK LETTER YOT
  "\u0456",             // CYRILLIC SMALL LETTER BYELORUSSIAN-UKRAINIAN I
  "\u0458",             // CYRILLIC SMALL LETTER JE
  "\u1D62",             // LATIN SUBSCRIPT SMALL LETTER I
  "\u1D96",             // LATIN SMALL LETTER I WITH RETROFLEX HOOK
  "\u1DA4",             // MODIFIER LETTER SMALL I WITH STROKE
  "\u1DA8",             // MODIFIER LETTER SMALL J WITH CROSSED-TAIL
  "\u1E2D",             // LATIN SMALL LETTER I WITH TILDE BELOW
  "\u1ECB",             // LATIN SMALL LETTER I WITH DOT BELOW
  "\u2071",             // SUPERSCRIPT LATIN SMALL LETTER I
  "\u2148", "\u2149",   // DOUBLE-STRUCK ITALIC SMALL I..DOUBLE-STRUCK ITALIC SMALL J
  "\u2C7C",             // LATIN SUBSCRIPT SMALL LETTER J
  "\uD835\uDC22", "\uD835\uDC23",   // MATHEMATICAL BOLD SMALL I..MATHEMATICAL BOLD SMALL J
  "\uD835\uDC56", "\uD835\uDC57",   // MATHEMATICAL ITALIC SMALL I..MATHEMATICAL ITALIC SMALL J
  "\uD835\uDC8A", "\uD835\uDC8B",   // MATHEMATICAL BOLD ITALIC SMALL I..MATHEMATICAL BOLD ITALIC SMALL J
  "\uD835\uDCBE", "\uD835\uDCBF",   // MATHEMATICAL SCRIPT SMALL I..MATHEMATICAL SCRIPT SMALL J
  "\uD835\uDCF2", "\uD835\uDCF3",   // MATHEMATICAL BOLD SCRIPT SMALL I..MATHEMATICAL BOLD SCRIPT SMALL J
  "\uD835\uDD26", "\uD835\uDD27",   // MATHEMATICAL FRAKTUR SMALL I..MATHEMATICAL FRAKTUR SMALL J
  "\uD835\uDD5A", "\uD835\uDD5B",   // MATHEMATICAL DOUBLE-STRUCK SMALL I..MATHEMATICAL DOUBLE-STRUCK SMALL J
  "\uD835\uDD8E", "\uD835\uDD8F",   // MATHEMATICAL BOLD FRAKTUR SMALL I..MATHEMATICAL BOLD FRAKTUR SMALL J
  "\uD835\uDDC2", "\uD835\uDDC3",   // MATHEMATICAL SANS-SERIF SMALL I..MATHEMATICAL SANS-SERIF SMALL J
  "\uD835\uDDF6", "\uD835\uDDF7",   // MATHEMATICAL SANS-SERIF BOLD SMALL I..MATHEMATICAL SANS-SERIF BOLD SMALL J
  "\uD835\uDE2A", "\uD835\uDE2B",   // MATHEMATICAL SANS-SERIF ITALIC SMALL I..MATHEMATICAL SANS-SERIF ITALIC SMALL J
  "\uD835\uDE5E", "\uD835\uDE5F",   // MATHEMATICAL SANS-SERIF BOLD ITALIC SMALL I..MATHEMATICAL SANS-SERIF BOLD ITALIC SMALL J
  "\uD835\uDE92", "\uD835\uDE93",   // MATHEMATICAL MONOSPACE SMALL I..MATHEMATICAL MONOSPACE SMALL J
];
assert.sameValue(softDotted.length, 46, "Total code points with Soft_Dotted property");

function charInfo(ch) {
  function hexString(n) {
    var s = n.toString(16).toUpperCase();
    return "0000".slice(s.length) + s;
  }

  if (ch.length === 1) {
    return "U+" + hexString(ch.charCodeAt(0));
  }
  var high = ch.charCodeAt(0);
  var low = ch.charCodeAt(1);
  var codePoint = ((high << 10) + low) + (0x10000 - (0xD800 << 10) - 0xDC00);
  return "U+" + hexString(codePoint) + " = " + hexString(high) + " " + hexString(low);
}


// COMBINING DOT ABOVE (U+0307) removed when preceded by Soft_Dotted.
// Character directly preceded by Soft_Dotted.
for (var i = 0; i < softDotted.length; ++i) {
  assert.sameValue(
    (softDotted[i] + "\u0307").toLocaleUpperCase("lt"),
    softDotted[i].toLocaleUpperCase("und"),
    "COMBINING DOT ABOVE preceded by Soft_Dotted (" + charInfo(softDotted[i]) + ")"
  );
}


// COMBINING DOT ABOVE (U+0307) removed if preceded by Soft_Dotted.
// Character not directly preceded by Soft_Dotted.
// - COMBINING DOT BELOW (U+0323), combining class 220 (Below)
for (var i = 0; i < softDotted.length; ++i) {
  assert.sameValue(
    (softDotted[i] + "\u0323\u0307").toLocaleUpperCase("lt"),
    softDotted[i].toLocaleUpperCase("und") + "\u0323",
    "COMBINING DOT ABOVE preceded by Soft_Dotted (" + charInfo(softDotted[i]) + "), COMBINING DOT BELOW"
  );
}


// COMBINING DOT ABOVE removed if preceded by Soft_Dotted.
// Character not directly preceded by Soft_Dotted.
// - PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE (U+101FD = D800 DDFD), combining class 220 (Below)
for (var i = 0; i < softDotted.length; ++i) {
  assert.sameValue(
    (softDotted[i] + "\uD800\uDDFD\u0307").toLocaleUpperCase("lt"),
    softDotted[i].toLocaleUpperCase("und") + "\uD800\uDDFD",
    "COMBINING DOT ABOVE preceded by Soft_Dotted (" + charInfo(softDotted[i]) + "), PHAISTOS DISC SIGN COMBINING OBLIQUE STROKE"
  );
}
