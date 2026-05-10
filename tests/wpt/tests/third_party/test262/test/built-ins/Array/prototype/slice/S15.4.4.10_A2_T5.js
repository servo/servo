// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The slice function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.slice
description: If end is undefined use length
---*/

var obj = {};
obj.slice = Array.prototype.slice;
obj[0] = 0;
obj[1] = 1;
obj[2] = 2;
obj[3] = 3;
obj[4] = 4;
obj.length = 5;
var arr = obj.slice(2);

arr.getClass = Object.prototype.toString;
if (arr.getClass() !== "[object " + "Array" + "]") {
  throw new Test262Error('#1: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(2); arr is Array object. Actual: ' + (arr.getClass()));
}

if (arr.length !== 3) {
  throw new Test262Error('#2: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(2); arr.length === 3. Actual: ' + (arr.length));
}

if (arr[0] !== 2) {
  throw new Test262Error('#3: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(2); arr[0] === 2. Actual: ' + (arr[0]));
}

if (arr[1] !== 3) {
  throw new Test262Error('#4: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(2); arr[1] === 3. Actual: ' + (arr[1]));
}

if (arr[2] !== 4) {
  throw new Test262Error('#5: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(2); arr[2] === 4. Actual: ' + (arr[2]));
}

if (arr[3] !== undefined) {
  throw new Test262Error('#6: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(2); arr[3] === undefined. Actual: ' + (arr[3]));
}
