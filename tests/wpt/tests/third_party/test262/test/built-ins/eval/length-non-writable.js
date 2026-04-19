// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of eval has the attribute ReadOnly
esid: sec-eval-x
description: Checking if varying the length property fails
includes: [propertyHelper.js]
---*/

//CHECK#1
var x = eval.length;
verifyNotWritable(eval, "length", null, Infinity);
if (eval.length !== x) {
  throw new Test262Error('#1: x = eval.length; eval.length = Infinity; eval.length === x. Actual: ' + (eval.length));
}
