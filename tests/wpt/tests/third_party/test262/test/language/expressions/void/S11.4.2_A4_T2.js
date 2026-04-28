// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator "void" evaluates UnaryExpression and returns undefined
es5id: 11.4.2_A4_T2
description: Type(x) is number primitive or Number object
---*/

//CHECK#1
var x = 0.1;
if (void x !== undefined) {
  throw new Test262Error('#1: var x = 0.1; void x === undefined. Actual: ' + (void x));
}

//CHECK#2
var x = new Number(-1.1);
if (void x !== undefined) {
  throw new Test262Error('#2: var x = new Number(-1.1); void x === undefined. Actual: ' + (void x));
}
