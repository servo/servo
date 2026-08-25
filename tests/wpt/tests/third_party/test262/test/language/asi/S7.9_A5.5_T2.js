// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check Function Expression for automatic semicolon insertion
es5id: 7.9_A5.5_T2
description: >
    Try use function f(o) {o.x = 1; return o;}; \n (new Object()).x;
    construction
---*/

//CHECK#1
var result = function f(o) {o.x = 1; return o;};
(new Object()).x;
if (typeof result !== "function") {
  throw new Test262Error('#1: Check Function Expression for automatic semicolon insertion');
}
