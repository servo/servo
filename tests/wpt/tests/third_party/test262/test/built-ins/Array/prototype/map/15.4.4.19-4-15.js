// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - calling with no callbackfn is the same as
    passing undefined for callbackfn
---*/

var obj = {
  10: 10
};
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
  Array.prototype.map.call(obj);
});
assert(lengthAccessed, 'lengthAccessed !== true');
assert.sameValue(loopAccessed, false, 'loopAccessed');
