// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: SPACE (U+0020) may occur within strings
es5id: 7.2_A2.4_T1
description: Use SPACE(\u0020)
---*/

// CHECK#1
if (eval("'\u0020str\u0020ing\u0020'") !== "\u0020str\u0020ing\u0020") {
  throw new Test262Error('#1: eval("\'\\u0020str\\u0020ing\\u0020\'") === "\\u0020str\\u0020ing\\u0020"');
}

//CHECK#2
if (eval("' str ing '") !== " str ing ") {
  throw new Test262Error('#2: eval("\' str ing \'") === " str ing "');
}
