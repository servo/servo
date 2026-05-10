// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-c-i-13
description: >
    Array.prototype.filter - element to be retrieved is own accessor
    property that overrides an inherited accessor property on an
    Array-like object
---*/

function callbackfn(val, idx, obj) {
  return idx === 1 && val === 12;
}

var proto = {};

Object.defineProperty(proto, "1", {
  get: function() {
    return 6;
  },
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 10;

Object.defineProperty(child, "1", {
  get: function() {
    return 12;
  },
  configurable: true
});
var newArr = Array.prototype.filter.call(child, callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 12, 'newArr[0]');
