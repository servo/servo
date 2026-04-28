// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Single line comment can contain SPACE (U+0020)
es5id: 7.2_A3.4_T2
description: Use real SPACE
---*/

//CHECK#1
var x = 0;
// single line comment x = 1;
if (x !== 0) {
  throw new Test262Error('#1: var x = 0; // single line comment x = 1; x === 0. Actual: ' + (x));
}
