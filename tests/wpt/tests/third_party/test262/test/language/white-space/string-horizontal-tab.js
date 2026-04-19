// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: HORIZONTAL TAB (U+0009) may occur within strings
es5id: 7.2_A2.1_T1
description: Use HORIZONTAL TAB(\u0009 and \t)
---*/

// CHECK#1
if (eval("'\u0009str\u0009ing\u0009'") !== "\u0009str\u0009ing\u0009") {
  throw new Test262Error('#1: eval("\'\\u0009str\\u0009ing\\u0009\'") === "\\u0009str\\u0009ing\\u0009"');
}

//CHECK#2
if (eval("'\tstr\ting\t'") !== "\tstr\ting\t") {
  throw new Test262Error('#2: eval("\'\\tstr\\ting\\t\'") === "\\tstr\\ting\\t"');
}
