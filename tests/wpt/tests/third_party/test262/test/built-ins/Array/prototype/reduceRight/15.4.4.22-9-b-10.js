// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - deleting property of prototype in
    step 8 causes deleted index property not to be visited on an
    Array-like Object
---*/

var accessed = false;
var testResult = true;

function callbackfn(preVal, val, idx, obj) {
  accessed = true;
  if (idx === 3) {
    testResult = false;
  }
}

var obj = {
  2: 2,
  length: 20
};

Object.defineProperty(obj, "5", {
  get: function() {
    delete Object.prototype[3];
    return 0;
  },
  configurable: true
});

Object.prototype[3] = 1;
Array.prototype.reduceRight.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
assert(accessed, 'accessed !== true');
