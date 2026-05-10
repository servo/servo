// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-RegularExpressionBackslashSequence
info: |
  RegularExpressionBackslashSequence ::
    \ RegularExpressionNonTerminator

  RegularExpressionNonTerminator ::
    SourceCharacter but not LineTerminator

    SyntaxError exception is thrown if the RegularExpressionNonTerminator position of a
    RegularExpressionBackslashSequence is a LineTerminator.
description: >
  A RegularExpressionBackslashSequence may not contain a LineTerminator.
---*/

assert.throws(SyntaxError, function() {
  eval("/\\\rn/;");
/*

The result of this string is:

"/\
n/;"

*/
});
