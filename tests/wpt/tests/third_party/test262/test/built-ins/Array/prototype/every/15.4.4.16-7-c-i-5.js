// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - element to be retrieved is own data
    property that overrides an inherited accessor property on an
    Array-like object
---*/

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    return val === 5;
  } else {
    return true;
  }
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

assert.sameValue(Array.prototype.every.call(child, callbackfn), false, 'Array.prototype.every.call(child, callbackfn)');
