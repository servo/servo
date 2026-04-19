// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The first element of the array is removed from the array and
    returned
esid: sec-array.prototype.shift
description: Checking this use new Array() and []
---*/

var x = new Array(0, 1, 2, 3);
var shift = x.shift();
if (shift !== 0) {
  throw new Test262Error('#1: x = new Array(0,1,2,3); x.shift() === 0. Actual: ' + (shift));
}

if (x.length !== 3) {
  throw new Test262Error('#2: x = new Array(0,1,2,3); x.shift(); x.length == 3');
}

if (x[0] !== 1) {
  throw new Test262Error('#3: x = new Array(0,1,2,3); x.shift(); x[0] == 1');
}

if (x[1] !== 2) {
  throw new Test262Error('#4: x = new Array(0,1,2,3); x.shift(); x[1] == 2');
}

x = [];
x[0] = 0;
x[3] = 3;
var shift = x.shift();
if (shift !== 0) {
  throw new Test262Error('#5: x = []; x[0] = 0; x[3] = 3; x.shift() === 0. Actual: ' + (shift));
}

if (x.length !== 3) {
  throw new Test262Error('#6: x = []; x[0] = 0; x[3] = 3; x.shift(); x.length == 3');
}

if (x[0] !== undefined) {
  throw new Test262Error('#7: x = []; x[0] = 0; x[3] = 3; x.shift(); x[0] == undefined');
}

if (x[12] !== undefined) {
  throw new Test262Error('#8: x = []; x[0] = 0; x[3] = 3; x.shift(); x[1] == undefined');
}

x.length = 1;
var shift = x.shift();
if (shift !== undefined) {
  throw new Test262Error('#9: x = []; x[0] = 0; x[3] = 3; x.shift(); x.length = 1; x.shift() === undefined. Actual: ' + (shift));
}

if (x.length !== 0) {
  throw new Test262Error('#10: x = []; x[0] = 0; x[3] = 3; x.shift(); x.length = 1; x.shift(); x.length === 0. Actual: ' + (x.length));
}
