// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - callbackfn that uses arguments object to
    get parameter value
---*/

var called = 0;

function callbackfn() {
  called++;
  return arguments[2][arguments[1]] === arguments[0];
}

assert([11, 12].every(callbackfn), '[11, 12].every(callbackfn) !== true');
assert.sameValue(called, 2, 'called');
