// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If length equal zero, call the [[Put]] method of this object
    with arguments "length" and 0 and return undefined
esid: sec-array.prototype.shift
description: Checking this algorithm
---*/

var x = new Array();
var shift = x.shift();
if (shift !== undefined) {
  throw new Test262Error('#1: var x = new Array(); x.shift() === undefined. Actual: ' + (shift));
}

if (x.length !== 0) {
  throw new Test262Error('#2: var x = new Array(); x.shift(); x.length === 0. Actual: ' + (x.length));
}

var x = Array(1, 2, 3);
x.length = 0;
var shift = x.shift();
if (shift !== undefined) {
  throw new Test262Error('#2: var x = Array(1,2,3); x.length = 0; x.shift() === undefined. Actual: ' + (shift));
}

if (x.length !== 0) {
  throw new Test262Error('#4: var x = new Array(1,2,3); x.length = 0; x.shift(); x.length === 0. Actual: ' + (x.length));
}
