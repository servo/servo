// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - element to be retrieved is own accessor
    property without a get function that overrides an inherited
    accessor property on an Array-like object
---*/

function callbackfn(val, idx, obj) {
  if (idx === 1) {
    return typeof val === "undefined";
  }
  return false;
}

var obj = {
  length: 2
};
Object.defineProperty(obj, "1", {
  set: function() {},
  configurable: true
});

Object.prototype[1] = 10;

assert(Array.prototype.some.call(obj, callbackfn), 'Array.prototype.some.call(obj, callbackfn) !== true');
