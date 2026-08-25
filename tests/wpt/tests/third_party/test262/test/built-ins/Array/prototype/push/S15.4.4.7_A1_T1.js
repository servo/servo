// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The arguments are appended to the end of the array, in
    the order in which they appear. The new length of the array is returned
    as the result of the call
esid: sec-array.prototype.push
description: Checking case when push is given no arguments or one argument
---*/

var x = new Array();
var push = x.push(1);
if (push !== 1) {
  throw new Test262Error('#1: x = new Array(); x.push(1) === 1. Actual: ' + (push));
}

if (x[0] !== 1) {
  throw new Test262Error('#2: x = new Array(); x.push(1); x[0] === 1. Actual: ' + (x[0]));
}

var push = x.push();
if (push !== 1) {
  throw new Test262Error('#3: x = new Array(); x.push(1); x.push() === 1. Actual: ' + (push));
}

if (x[1] !== undefined) {
  throw new Test262Error('#4: x = new Array(); x.push(1); x.push(); x[1] === unedfined. Actual: ' + (x[1]));
}

var push = x.push(-1);
if (push !== 2) {
  throw new Test262Error('#5: x = new Array(); x.push(1); x.push(); x.push(-1) === 2. Actual: ' + (push));
}

if (x[1] !== -1) {
  throw new Test262Error('#6: x = new Array(); x.push(1); x.push(-1); x[1] === -1. Actual: ' + (x[1]));
}

if (x.length !== 2) {
  throw new Test262Error('#7: x = new Array(); x.push(1); x.push(); x.push(-1); x.length === 2. Actual: ' + (x.length));
}
