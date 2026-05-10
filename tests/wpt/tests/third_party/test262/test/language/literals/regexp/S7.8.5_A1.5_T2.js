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
  A RegularExpressionBackslashSequence may not contain a <LF>, evaluated

---*/

//CHECK#1
try {
   eval("/\\\u000A/").source;
   throw new Test262Error('#1.1: RegularExpressionFirstChar :: BackslashSequence :: \\Line Feed is incorrect. Actual: ' + (eval("/\\\u000A/").source));
}
catch (e) {
  if ((e instanceof SyntaxError) !== true) {
     throw new Test262Error('#1.2: RegularExpressionFirstChar :: BackslashSequence :: \\Line Feed is incorrect. Actual: ' + (e));
  }
}
