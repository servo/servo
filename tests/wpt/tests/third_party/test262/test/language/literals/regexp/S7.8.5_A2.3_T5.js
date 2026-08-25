// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-RegularExpressionNonTerminator
info: |
  RegularExpressionBody ::
    RegularExpressionFirstChar RegularExpressionChars

  RegularExpressionChars ::
    [empty]
    RegularExpressionChars RegularExpressionChar

  RegularExpressionFirstChar ::
    RegularExpressionNonTerminator but not one of * or \ or / or [

  RegularExpressionNonTerminator ::
    SourceCharacter but not LineTerminator

description: >
  A regular expression may not contain a <LS> as a SourceCharacter, evaluated

---*/

//CHECK#1
try {
   eval("/a\u2028/").source;
   throw new Test262Error('#1.1: RegularExpressionChar :: Line separator is incorrect. Actual: ' + (eval("/a\u2028/").source));
}
catch (e) {
  if ((e instanceof SyntaxError) !== true) {
     throw new Test262Error('#1.2: RegularExpressionChar :: Line separator is incorrect. Actual: ' + (e));
  }
}
