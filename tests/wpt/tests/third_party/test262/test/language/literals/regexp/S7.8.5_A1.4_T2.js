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
  var Elimination =
    ((cu === 0x002A) || (cu === 0x002F) || (cu === 0x005C) || (cu === 0x002B) ||
     (cu === 0x003F) || (cu === 0x0028) || (cu === 0x0029) ||
     (cu === 0x005B) || (cu === 0x005D) || (cu === 0x007B) || (cu === 0x007D));
     /*
          * \u002A     / \u002F     \ \u005C     + \u002B
          ? \u003F     ( \u0028     ) \u0029
          [ \u005B     ] \u005D     { \u007B     } \u007D
     */
  var LineTerminator = ((cu === 0x000A) || (cu === 0x000D) || (cu === 0x2028) || (cu === 0x2029));
  if ((Elimination || LineTerminator ) === false) {
    var xx = "\\" + String.fromCharCode(cu);
    try {
      var pattern = eval("/" + xx + "/");
    } catch (e) {
      var identifierPartNotUnicodeIDContinue = ((cu === 0x0024) || (cu === 0x200C) || (cu === 0x200D));
      if (e instanceof SyntaxError && !identifierPartNotUnicodeIDContinue) {
        // Use eval with var-declaration to check if `cu` is in UnicodeIDContinue.
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
