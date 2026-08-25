// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - 'length' is inherited accessor
    property on an Array-like object
---*/

var proto = {};
Object.defineProperty(proto, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child[1] = 1;
child[2] = 2;

assert.sameValue(Array.prototype.lastIndexOf.call(child, 1), 1, 'Array.prototype.lastIndexOf.call(child, 1)');
assert.sameValue(Array.prototype.lastIndexOf.call(child, 2), -1, 'Array.prototype.lastIndexOf.call(child, 2)');
