// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - 'length' is an inherited data
    property on an Array-like object
---*/

var proto = {
  length: 2
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child[1] = "x";
child[2] = "y";

assert.sameValue(Array.prototype.lastIndexOf.call(child, "x"), 1, 'Array.prototype.lastIndexOf.call(child, "x")');
assert.sameValue(Array.prototype.lastIndexOf.call(child, "y"), -1, 'Array.prototype.lastIndexOf.call(child, "y")');
