// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The pop function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.pop
description: >
    If ToUint32(length) equal zero, call the [[Put]] method  of this
    object with arguments "length" and 0 and return undefined
---*/

var obj = {};
obj.pop = Array.prototype.pop;

if (obj.length !== undefined) {
  throw new Test262Error('#0: var obj = {}; obj.length === undefined. Actual: ' + (obj.length));
} else {
  var pop = obj.pop();
  if (pop !== undefined) {
    throw new Test262Error('#1: var obj = {}; obj.pop = Array.prototype.pop; obj.pop() === undefined. Actual: ' + (pop));
  }
  if (obj.length !== 0) {
    throw new Test262Error('#2: var obj = {}; obj.pop = Array.prototype.pop; obj.pop(); obj.length === 0. Actual: ' + (obj.length));
  }
}

obj.length = undefined;
var pop = obj.pop();
if (pop !== undefined) {
  throw new Test262Error('#3: var obj = {}; obj.length = undefined; obj.pop = Array.prototype.pop; obj.pop() === undefined. Actual: ' + (pop));
}

if (obj.length !== 0) {
  throw new Test262Error('#4: var obj = {}; obj.length = undefined; obj.pop = Array.prototype.pop; obj.pop(); obj.length === 0. Actual: ' + (obj.length));
}

obj.length = null
var pop = obj.pop();
if (pop !== undefined) {
  throw new Test262Error('#5: var obj = {}; obj.length = null; obj.pop = Array.prototype.pop; obj.pop() === undefined. Actual: ' + (pop));
}

if (obj.length !== 0) {
  throw new Test262Error('#6: var obj = {}; obj.length = null; obj.pop = Array.prototype.pop; obj.pop(); obj.length === 0. Actual: ' + (obj.length));
}
