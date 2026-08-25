// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - 'length' is inherited accessor property
    without a get function on an Array-like object
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return val > 10;
}

var proto = {};
Object.defineProperty(proto, "length", {
  set: function() {},
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child[0] = 11;
child[1] = 12;

assert.sameValue(Array.prototype.some.call(child, callbackfn), false, 'Array.prototype.some.call(child, callbackfn)');
assert.sameValue(accessed, false, 'accessed');
