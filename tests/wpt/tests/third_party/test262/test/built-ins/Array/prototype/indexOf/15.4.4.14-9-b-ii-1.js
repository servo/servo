// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - type of array element is different from
    type of search element
---*/

assert.sameValue(["true"].indexOf(true), -1, '["true"].indexOf(true)');
assert.sameValue(["0"].indexOf(0), -1, '["0"].indexOf(0)');
assert.sameValue([false].indexOf(0), -1, '[false].indexOf(0)');
assert.sameValue([undefined].indexOf(0), -1, '[undefined].indexOf(0)');
assert.sameValue([null].indexOf(0), -1, '[null].indexOf(0)');
assert.sameValue([
  []
].indexOf(0), -1, '[[]].indexOf(0)');
