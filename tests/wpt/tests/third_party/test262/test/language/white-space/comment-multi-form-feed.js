// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Multi line comment can contain FORM FEED (U+000C)
es5id: 7.2_A4.3_T1
description: Use FORM FEED(\u000C)
---*/

// CHECK#1
eval("/*\u000C multi line \u000C comment \u000C*/");

//CHECK#2
var x = 0;
eval("/*\u000C multi line \u000C comment \u000C x = 1;*/");
if (x !== 0) {
  throw new Test262Error('#1: var x = 0; eval("/*\\u000C multi line \\u000C comment \\u000C x = 1;*/"); x === 0. Actual: ' + (x));
}
