// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-RegularExpressionBackslashSequence
info: |
  RegularExpressionBackslashSequence ::
  \ RegularExpressionNonTerminator

  RegularExpressionNonTerminator ::
    SourceCharacter but not LineTerminator

description: >
  A RegularExpressionBackslashSequence may not contain a <CR>, evaluated
---*/

//CHECK#1
try {
   eval("/\\\u000D/").source;
   throw new Test262Error('#1.1: RegularExpressionFirstChar :: BackslashSequence :: \\Carriage Return is incorrect. Actual: ' + (eval("/\\\u000D/").source));
}
catch (e) {
  if ((e instanceof SyntaxError) !== true) {
     throw new Test262Error('#1.2: RegularExpressionFirstChar :: BackslashSequence :: \\Carriage Return is incorrect. Actual: ' + (e));
  }
}
