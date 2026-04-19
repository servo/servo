// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - type of array element is different
    from type of search element
---*/

assert.sameValue(["true"].lastIndexOf(true), -1, '["true"].lastIndexOf(true)');
assert.sameValue(["0"].lastIndexOf(0), -1, '["0"].lastIndexOf(0)');
assert.sameValue([false].lastIndexOf(0), -1, '[false].lastIndexOf(0)');
assert.sameValue([undefined].lastIndexOf(0), -1, '[undefined].lastIndexOf(0)');
assert.sameValue([null].lastIndexOf(0), -1, '[null].lastIndexOf(0)');
assert.sameValue([
  []
].lastIndexOf(0), -1, '[[]].lastIndexOf(0)');
