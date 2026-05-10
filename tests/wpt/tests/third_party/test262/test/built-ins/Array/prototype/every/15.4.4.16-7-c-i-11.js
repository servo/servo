// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - element to be retrieved is own accessor
    property that overrides an inherited data property on an
    Array-like object
---*/

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    return val === 5;
  } else {
    return true;
  }
}

var proto = {
  0: 5,
  1: 6
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 10;

Object.defineProperty(child, "0", {
  get: function() {
    return 11;
  },
  configurable: true
});

assert.sameValue(Array.prototype.every.call(child, callbackfn), false, 'Array.prototype.every.call(child, callbackfn)');
