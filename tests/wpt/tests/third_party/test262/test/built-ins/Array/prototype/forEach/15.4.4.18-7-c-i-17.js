// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - element to be retrieved is own accessor
    property without a get function on an Array-like object
---*/

var testResult = false;

function callbackfn(val, idx, obj) {
  if (idx === 1) {
    testResult = (typeof val === "undefined");
  }
}

var obj = {
  length: 2
};
Object.defineProperty(obj, "1", {
  set: function() {},
  configurable: true
});

Array.prototype.forEach.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
