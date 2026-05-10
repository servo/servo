// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - element to be retrieved is own accessor
    property that overrides an inherited accessor property on an
    Array-like object
---*/

function callbackfn(val, idx, obj) {
  if (idx === 1) {
    return val === 6;
  } else {
    return true;
  }
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


assert.sameValue(Array.prototype.every.call(child, callbackfn), false, 'Array.prototype.every.call(child, callbackfn)');
