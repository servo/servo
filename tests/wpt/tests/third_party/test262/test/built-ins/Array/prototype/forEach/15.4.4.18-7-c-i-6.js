// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - element to be retrieved is own data
    property that overrides an inherited accessor property on an Array
---*/

var testResult = false;

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    testResult = (val === 11);
  }
}

Object.defineProperty(Array.prototype, "0", {
  get: function() {
    return 9;
  },
  configurable: true
});

[11].forEach(callbackfn);

assert(testResult, 'testResult !== true');
