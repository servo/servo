// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - element to be retrieved is own data
    property on an Array
---*/

assert.sameValue([true, true, true].indexOf(true), 0, '[true, true, true].indexOf(true)');
assert.sameValue([false, true, true].indexOf(true), 1, '[false, true, true].indexOf(true)');
assert.sameValue([false, false, true].indexOf(true), 2, '[false, false, true].indexOf(true)');
