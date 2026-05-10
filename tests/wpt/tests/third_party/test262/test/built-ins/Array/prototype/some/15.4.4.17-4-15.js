// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - calling with no callbackfn is the same as
    passing undefined for callbackfn
---*/

var obj = {};
var lengthAccessed = false;
var loopAccessed = false;

Object.defineProperty(obj, "length", {
  get: function() {
    lengthAccessed = true;
    return 20;
  },
  configurable: true
});
Object.defineProperty(obj, "0", {
  get: function() {
    loopAccessed = true;
    return 10;
  },
  configurable: true
});
assert.throws(TypeError, function() {
  Array.prototype.some.call(obj);
});
assert(lengthAccessed, 'lengthAccessed !== true');
assert.sameValue(loopAccessed, false, 'loopAccessed');
