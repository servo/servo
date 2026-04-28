// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If start is positive, use min(start, length).
    If deleteCount is negative, use 0
esid: sec-array.prototype.splice
description: -length < deleteCount < start = 0, itemCount = 0
---*/

var x = [0, 1];
var arr = x.splice(0, -1);

arr.getClass = Object.prototype.toString;
if (arr.getClass() !== "[object " + "Array" + "]") {
  throw new Test262Error('#0: var x = [0,1]; var arr = x.splice(0,-1); arr is Array object. Actual: ' + (arr.getClass()));
}

if (arr.length !== 0) {
  throw new Test262Error('#1: var x = [0,1]; var arr = x.splice(0,-1); arr.length === 0. Actual: ' + (arr.length));
}

if (x.length !== 2) {
  throw new Test262Error('#2: var x = [0,1]; var arr = x.splice(0,-1); x.length === 2. Actual: ' + (x.length));
}

if (x[0] !== 0) {
  throw new Test262Error('#3: var x = [0,1]; var arr = x.splice(0,-1); x[0] === 0. Actual: ' + (x[0]));
}

if (x[1] !== 1) {
  throw new Test262Error('#4: var x = [0,1]; var arr = x.splice(0,-1); x[1] === 1. Actual: ' + (x[1]));
}
