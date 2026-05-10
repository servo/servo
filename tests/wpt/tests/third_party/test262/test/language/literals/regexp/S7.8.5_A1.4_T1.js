// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegularExpressionFirstChar :: BackslashSequence :: \NonTerminator,
    RegularExpressionChars :: [empty], RegularExpressionFlags :: [empty]
es5id: 7.8.5_A1.4_T1
description: Check similar to (/\;/.source === "\\;")
---*/

//CHECK#1
if (/\;/.source !== "\\;") {
  throw new Test262Error('#1: /\\;/');
}

//CHECK#2
if (/\ /.source !== "\\ ") {
  throw new Test262Error('#2: /\\ /');
}
