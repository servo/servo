// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.slice
description: Array.prototype.slice applied to boolean primitive
includes: [compareArray.js]
---*/

assert.compareArray(Array.prototype.slice.call(true), [], 'Array.prototype.slice.call(true) must return []');
assert.compareArray(Array.prototype.slice.call(false), [], 'Array.prototype.slice.call(false) must return []');
