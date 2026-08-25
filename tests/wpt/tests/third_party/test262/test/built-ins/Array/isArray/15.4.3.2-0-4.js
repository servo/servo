// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.isarray
description: Array.isArray return false if its argument is not an Array
---*/

assert.sameValue(Array.isArray(42), false, 'Array.isArray(42) must return false');
assert.sameValue(Array.isArray(undefined), false, 'Array.isArray(undefined) must return false');
assert.sameValue(Array.isArray(true), false, 'Array.isArray(true) must return false');
assert.sameValue(Array.isArray("abc"), false, 'Array.isArray("abc") must return false');
assert.sameValue(Array.isArray({}), false, 'Array.isArray({}) must return false');
assert.sameValue(Array.isArray(null), false, 'Array.isArray(null) must return false');
