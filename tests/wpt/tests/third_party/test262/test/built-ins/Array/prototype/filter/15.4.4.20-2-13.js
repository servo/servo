// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter applied to the Array-like object that
    'length' is inherited accessor property without a get function
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return true;
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

var newArr = Array.prototype.filter.call(child, callbackfn);

assert.sameValue(newArr.length, 0, 'newArr.length');
assert.sameValue(accessed, false, 'accessed');
