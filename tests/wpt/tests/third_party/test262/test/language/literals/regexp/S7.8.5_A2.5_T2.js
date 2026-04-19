// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-RegularExpressionBackslashSequence
info: |
  RegularExpressionBackslashSequence ::
    \ RegularExpressionNonTerminator

  RegularExpressionNonTerminator ::
    SourceCharacter but not LineTerminator

  LineTerminator ::
    <LF>
    <CR>
    <LS>
    <PS>

description: >
  A regular expression may not contain a <LF> as a SourceCharacter, evaluated
---*/

//CHECK#1
try {
   eval("/a\\\u000A/").source;
   throw new Test262Error('#1.1: RegularExpressionChar :: BackslashSequence :: \\Line Feed is incorrect. Actual: ' + (eval("/a\\\u000A/").source));
}
catch (e) {
  if ((e instanceof SyntaxError) !== true) {
     throw new Test262Error('#1.2: RegularExpressionChar :: BackslashSequence :: \\Line Feed is incorrect. Actual: ' + (e));
  }
}
