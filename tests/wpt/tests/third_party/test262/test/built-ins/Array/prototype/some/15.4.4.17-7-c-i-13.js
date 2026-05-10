// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - element to be retrieved is own accessor
    property that overrides an inherited accessor property on an
    Array-like object
---*/

var kValue = "abc";

function callbackfn(val, idx, obj) {
  if (idx === 1) {
    return val === kValue;
  }
  return false;
}

var proto = {};

Object.defineProperty(proto, "1", {
  get: function() {
    return 5;
  },
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 10;

Object.defineProperty(child, "1", {
  get: function() {
    return kValue;
  },
  configurable: true
});


assert(Array.prototype.some.call(child, callbackfn), 'Array.prototype.some.call(child, callbackfn) !== true');
