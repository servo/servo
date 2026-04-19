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
  A regular expression may not contain a <PS> as a SourceCharacter, evaluated

---*/

try {
   eval("/s\u2029/").source;
   throw new Test262Error('#1.1: RegularExpressionChar :: Paragraph separator is incorrect. Actual: ' + (eval("/s\u2029/").source));
}
catch (e) {
  if ((e instanceof SyntaxError) !== true) {
     throw new Test262Error('#1.2: RegularExpressionChar :: Paragraph separator is incorrect. Actual: ' + (e));
  }
}
