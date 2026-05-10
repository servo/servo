// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The join function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.join
description: If ToUint32(length) is zero, return the empty string
---*/

var obj = {};
obj.join = Array.prototype.join;

obj.length = 4.5;
if (obj.join() !== ",,,") {
  throw new Test262Error('#1: var obj = {}; obj.length = 4.5; obj.join = Array.prototype.join; obj.join() === ",,,". Actual: ' + (obj.join()));
}

obj[0] = undefined;
obj[1] = 1;
obj[2] = null;
if (obj.join() !== ",1,,") {
  throw new Test262Error('#1: var obj = {}; obj.length = 4.5; obj[0] = undefined; obj[1] = 1; obj[2] = null; obj.join = Array.prototype.join; obj.join() === ",1,,". Actual: ' + (obj.join()));
}

if (obj.length !== 4.5) {
  throw new Test262Error('#1: var obj = {}; obj.length = 4.5; obj[0] = undefined; obj[1] = 1; obj[2] = null; obj.join = Array.prototype.join; obj.join(); obj.length === 4.5. Actual: ' + (obj.length));
}

var obj = {};
obj.join = Array.prototype.join;

var x = new Number(4.5);
obj.length = x;
if (obj.join() !== ",,,") {
  throw new Test262Error('#4: var obj = {}; var x = new Number(4.5); obj.length = x; obj.join = Array.prototype.join; obj.join() === ",,,". Actual: ' + (obj.join()));
}

obj[0] = undefined;
obj[1] = 1;
obj[2] = null;
if (obj.join() !== ",1,,") {
  throw new Test262Error('#5: var obj = {}; var x = new Number(4.5); obj.length = x; obj[0] = undefined; obj[1] = 1; obj[2] = null; obj.join = Array.prototype.join; obj.join() === ",1,,". Actual: ' + (obj.join()));
}

if (obj.length !== x) {
  throw new Test262Error('#6: var obj = {}; var x = new Number(4.5); obj.length = x; obj[0] = undefined; obj[1] = 1; obj[2] = null; obj.join = Array.prototype.join; obj.join(); obj.length === x. Actual: ' + (obj.length));
}
