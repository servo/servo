// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - deleting own property causes index property
    not to be visited on an Array-like object
---*/

function callbackfn(val, idx, obj) {
  if (idx === 1) {
    return false;
  } else {
    return true;
  }
}
var obj = {
  length: 2
};

Object.defineProperty(obj, "1", {
  get: function() {
    return 6.99;
  },
  configurable: true
});

Object.defineProperty(obj, "0", {
  get: function() {
    delete obj[1];
    return 0;
  },
  configurable: true
});

var testResult = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(testResult[0], true, 'testResult[0]');
assert.sameValue(typeof testResult[1], "undefined", 'typeof testResult[1]');
