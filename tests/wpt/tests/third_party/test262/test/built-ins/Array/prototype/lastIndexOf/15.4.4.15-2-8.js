// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - 'length' is own accessor property
    that overrides an inherited data property on an Array-like object
---*/

var proto = {
  length: 0
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child[1] = eval;

Object.defineProperty(child, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

assert.sameValue(Array.prototype.lastIndexOf.call(child, eval), 1, 'Array.prototype.lastIndexOf.call(child, eval)');
