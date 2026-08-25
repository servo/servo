// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.pop
description: Array.prototype.pop applied to boolean primitive
---*/

assert.sameValue(Array.prototype.pop.call(true), undefined, 'Array.prototype.pop.call(true) must return undefined');
assert.sameValue(Array.prototype.pop.call(false), undefined, 'Array.prototype.pop.call(false) must return undefined');
