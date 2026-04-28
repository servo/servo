// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - element to be retrieved is own data
    property that overrides an inherited accessor property on an
    Array-like object
---*/

function callbackfn(val, idx, obj) {
  return idx === 0 && val === 11;
}

var proto = {};

Object.defineProperty(proto, "0", {
  get: function() {
    return 5;
  },
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 2;
Object.defineProperty(child, "0", {
  value: 11,
  configurable: true
});
child[1] = 12;

var newArr = Array.prototype.filter.call(child, callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 11, 'newArr[0]');
