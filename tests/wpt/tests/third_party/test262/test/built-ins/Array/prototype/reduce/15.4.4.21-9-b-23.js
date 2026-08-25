// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - deleting property of prototype causes
    deleted index property not to be visited on an Array-like Object
---*/

var accessed = false;
var testResult = true;

function callbackfn(accum, val, idx, obj) {
  accessed = true;
  if (idx === 3) {
    testResult = false;
  }
}

var obj = {
  2: 2,
  length: 20
};

Object.defineProperty(obj, "0", {
  get: function() {
    delete Object.prototype[3];
    return 0;
  },
  configurable: true
});

Object.prototype[3] = 1;
Array.prototype.reduce.call(obj, callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
assert(accessed, 'accessed !== true');
