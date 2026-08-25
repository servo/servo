// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - deleted properties in step 2 are visible
    here
---*/

var accessed = false;
var testResult = true;

function callbackfn(accum, val, idx, obj) {
  accessed = true;
  if (idx === 2) {
    testResult = false;
  }
}

var obj = {
  2: "2",
  3: 10
};

Object.defineProperty(obj, "length", {
  get: function() {
    delete obj[2];
    return 5;
  },
  configurable: true
});

Array.prototype.reduce.call(obj, callbackfn, "initialValue");

assert(accessed, 'accessed !== true');
assert(testResult, 'testResult !== true');
