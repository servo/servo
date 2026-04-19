// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - 'length' is own accessor property
    without a get function that overrides an inherited accessor
    property on an Array
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
}

Object.defineProperty(Object.prototype, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

var obj = {
  0: 12,
  1: 11
};
Object.defineProperty(obj, "length", {
  set: function() {},
  configurable: true
});

Array.prototype.forEach.call(obj, callbackfn);

assert.sameValue(accessed, false, 'accessed');
