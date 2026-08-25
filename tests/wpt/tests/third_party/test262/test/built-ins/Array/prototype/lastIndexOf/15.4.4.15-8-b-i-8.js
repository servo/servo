// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - element to be retrieved is inherited
    data property on an Array-like object
---*/

Object.prototype[0] = true;
Object.prototype[1] = false;
Object.prototype[2] = "true";

assert.sameValue(Array.prototype.lastIndexOf.call({
  length: 3
}, true), 0, 'Array.prototype.lastIndexOf.call({ length: 3 }, true)');
assert.sameValue(Array.prototype.lastIndexOf.call({
  length: 3
}, false), 1, 'Array.prototype.lastIndexOf.call({ length: 3 }, false)');
assert.sameValue(Array.prototype.lastIndexOf.call({
  length: 3
}, "true"), 2, 'Array.prototype.lastIndexOf.call({ length: 3 }, "true")');
