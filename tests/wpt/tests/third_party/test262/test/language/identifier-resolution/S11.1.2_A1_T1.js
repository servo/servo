// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The result of evaluating an Identifier is always a value of type Reference
es5id: 11.1.2_A1_T1
description: Creating variables without defining it
---*/

//CHECK#1
if (this.x !== undefined) {
  throw new Test262Error('#1: this.x === undefined. Actual: ' + (this.x));
}

//CHECK#2
var object = new Object();
if (object.prop !== undefined) {
  throw new Test262Error('#2: var object = new Object(); object.prop === undefined. Actual: ' + (object.prop));
}

//CHECK#3
this.y++;
if (isNaN(y) !== true) {
  throw new Test262Error('#3: this.y++; y === Not-a-Number. Actual: ' + (y));
}
