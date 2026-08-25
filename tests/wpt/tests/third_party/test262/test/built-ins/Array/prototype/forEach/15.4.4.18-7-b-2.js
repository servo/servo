// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - added properties in step 2 are visible
    here
---*/

var testResult = false;

function callbackfn(val, idx, obj) {
  if (idx === 2 && val === "length") {
    testResult = true;
  }
}

var obj = {};

Object.defineProperty(obj, "length", {
  get: function() {
    obj[2] = "length";
    return 3;
  },
  configurable: true
});

Array.prototype.forEach.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
