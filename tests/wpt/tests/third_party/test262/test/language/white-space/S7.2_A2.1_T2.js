// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: HORIZONTAL TAB (U+0009) may occur within strings
es5id: 7.2_A2.1_T2
description: Use real HORIZONTAL TAB
---*/

//CHECK#1
if ("	str	ing	" !== "\u0009str\u0009ing\u0009") {
  throw new Test262Error('#1: "	str	ing	" === "\\u0009str\\u0009ing\\u0009"');
}
