// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "EscapeSequence :: 0"
es5id: 7.8.4_A5.1_T2
description: "\"\\u0000\""
---*/

//CHECK#1
if ("\u0000" !== "\0") {
  throw new Test262Error('#1: "\\u0000" === "\\0"');
}
