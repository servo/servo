// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every applied to the Array-like object that
    'length' is inherited accessor property without a get function
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
child[0] = 9;
child[1] = 8;

assert(Array.prototype.every.call(child, callbackfn), 'Array.prototype.every.call(child, callbackfn) !== true');
assert.sameValue(accessed, false, 'accessed');
