// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator ~x uses GetValue
es5id: 11.4.8_A2.1_T1
description: Either Type(x) is not Reference or GetBase(x) is not null
---*/

//CHECK#1
if (~0 !== -1) {
  throw new Test262Error('#1: ~0 === -1. Actual: ' + (~0));
}

//CHECK#2
if (~(~0) !== 0) {
  throw new Test262Error('#2: ~(~0) === 0. Actual: ' + (~(~0)));
}

//CHECK#3
var x = 0;
if (~x !== -1) {
  throw new Test262Error('#3: var x = 0; ~x === -1. Actual: ' + (~x));
}

//CHECK#4
var x = 0;
if (~(~x) !== 0) {
  throw new Test262Error('#4: var x = 0; ~(~x) === 0. Actual: ' + (~(~x)));
}

//CHECK#5
var object = new Object();
object.prop = 0;
if (~object.prop !== -1) {
  throw new Test262Error('#5: var object = new Object(); object.prop = 0; ~object.prop === -1. Actual: ' + (~object.prop));
}
