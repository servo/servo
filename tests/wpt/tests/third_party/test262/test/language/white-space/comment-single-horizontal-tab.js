// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Single line comment can contain HORIZONTAL TAB (U+0009)
es5id: 7.2_A3.1_T1
description: Use HORIZONTAL TAB(\u0009)
---*/

// CHECK#1
eval("//\u0009 single line \u0009 comment \u0009");

//CHECK#2
var x = 0;
eval("//\u0009 single line \u0009 comment \u0009 x = 1;");
if (x !== 0) {
  throw new Test262Error('#1: var x = 0; eval("//\\u0009 single line \\u0009 comment \\u0009 x = 1;"); x === 0. Actual: ' + (x));
}
