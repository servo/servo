// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach applied to Array-like object, 'length' is
    an own accessor property without a get function
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
}

var obj = {
  0: 11,
  1: 12
};
Object.defineProperty(obj, "length", {
  set: function() {},
  configurable: true
});

Array.prototype.forEach.call(obj, callbackfn);

assert.sameValue(accessed, false, 'accessed');
