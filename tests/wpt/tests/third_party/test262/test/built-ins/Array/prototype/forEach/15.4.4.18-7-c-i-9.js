// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - element to be retrieved is own accessor
    property on an Array-like object
---*/

var testResult = false;

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    testResult = (val === 11);
  }
}

var obj = {
  10: 10,
  length: 20
};

Object.defineProperty(obj, "0", {
  get: function() {
    return 11;
  },
  configurable: true
});

Array.prototype.forEach.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
