// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If the parse fails, throw a SyntaxError exception (but see also clause 16)
es5id: 15.1.2.1_A2_T2
description: Checking if execution of "eval("x = 1; x\u000A++")" fails
---*/

var x;
assert.throws(SyntaxError, function() {
  eval("x = 1; x\u000A++");
});
