// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - element to be retrieved is own accessor
    property without a get function on an Array-like object
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return typeof val === "undefined";
}

var obj = {
  length: 2
};
Object.defineProperty(obj, "1", {
  set: function() {},
  configurable: true
});

assert(Array.prototype.every.call(obj, callbackfn), 'Array.prototype.every.call(obj, callbackfn) !== true');
assert(accessed, 'accessed !== true');
