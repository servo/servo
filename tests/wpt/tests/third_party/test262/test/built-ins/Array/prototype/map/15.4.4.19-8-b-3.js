// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - deleted properties in step 2 are visible here
---*/

function callbackfn(val, idx, obj) {
  if (idx === 2) {
    return false;
  } else {
    return true;
  }
}
var obj = {
  2: 6.99,
  8: 19
};

Object.defineProperty(obj, "length", {
  get: function() {
    delete obj[2];
    return 10;
  },
  configurable: true
});

var testResult = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(typeof testResult[2], "undefined", 'typeof testResult[2]');
