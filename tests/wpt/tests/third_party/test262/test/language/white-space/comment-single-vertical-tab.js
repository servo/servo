// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Single line comment can contain VERTICAL TAB (U+000B)
es5id: 7.2_A3.2_T1
description: Use VERTICAL TAB(\u000B)
---*/

// CHECK#1
eval("//\u000B single line \u000B comment \u000B");

//CHECK#2
var x = 0;
eval("//\u000B single line \u000B comment \u000B x = 1;");
if (x !== 0) {
  throw new Test262Error('#1: var x = 0; eval("//\\u000B single line \\u000B comment \\u000B x = 1;"); x === 0. Actual: ' + (x));
}
