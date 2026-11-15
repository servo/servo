// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegularExpressionFirstChar :: BackslashSequence :: \NonTerminator,
    RegularExpressionChars :: [empty], RegularExpressionFlags :: [empty]
es5id: 7.8.5_A1.4_T2
description: Complex test with eval, using syntax pattern
---*/

for (var cu = 0; cu <= 0xffff; ++cu) {
  var LineTerminator = ((cu === 0x000A) || (cu === 0x000D) || (cu === 0x2028) || (cu === 0x2029));
  if (LineTerminator === false) {
    var xx = "\\" + String.fromCharCode(cu);
    try {
      var pattern = eval("/" + xx + "/");
    } catch (e) {
      // If `cu` is in |UnicodeIDContinue|, a SyntaxError is acceptable
      // (but not required, cf. Annex B |SourceCharacterIdentityEscape|).
      // Check by using it in an eval-declared `var`.
      var isSpecialIdentifierPart = cu === 0x0024; // `$` is not ID_Continue
      if (e instanceof SyntaxError && !isSpecialIdentifierPart) {
        try {
          eval("var _" + String.fromCharCode(cu));
          continue;
        } catch (ignore) { }
      }
      throw e;
    }
    assert.sameValue(pattern.source, xx, "Code unit: " + cu.toString(16));
  }
}
