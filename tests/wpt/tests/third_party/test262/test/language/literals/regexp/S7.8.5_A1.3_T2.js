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
  The first character of a regular expression may not be a <LF> (\u000A), evaluated
---*/

//CHECK#1
try {
   eval("/\u000A/").source;
   throw new Test262Error('#1.1: RegularExpressionFirstChar :: Line Feed is incorrect. Actual: ' + (eval("/\u000A/").source));
}
catch (e) {
  if ((e instanceof SyntaxError) !== true) {
     throw new Test262Error('#1.2: RegularExpressionFirstChar :: Line Feed is incorrect. Actual: ' + (e));
  }
}
