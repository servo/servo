// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator "void" evaluates UnaryExpression and returns undefined
es5id: 11.4.2_A4_T1
description: Type(x) is boolean primitive or Boolean object
---*/

//CHECK#1
var x = false; 
if (void x !== undefined) {
  throw new Test262Error('#1: var x = false; void x === undefined. Actual: ' + (void x));
}

//CHECK#2
var x = new Boolean(true);
if (void x !== undefined) {
  throw new Test262Error('#2: var x = new Boolean(true); void x === undefined. Actual: ' + (void x));
}
