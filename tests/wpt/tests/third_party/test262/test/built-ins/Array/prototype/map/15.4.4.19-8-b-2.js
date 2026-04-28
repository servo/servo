// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - added properties in step 2 are visible here
---*/

function callbackfn(val, idx, obj) {
  if (idx === 2 && val === "length") {
    return false;
  } else {
    return true;
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

var testResult = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(testResult[2], false, 'testResult[2]');
