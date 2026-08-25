// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If R < 2 or R > 36, then return NaN
esid: sec-parseint-string-radix
description: R = 1
---*/

assert.sameValue(parseInt("0", 1), NaN, 'parseInt("0", 1) must return NaN');
assert.sameValue(parseInt("1", 1), NaN, 'parseInt("1", 1) must return NaN');
assert.sameValue(parseInt("2", 1), NaN, 'parseInt("2", 1) must return NaN');
assert.sameValue(parseInt("3", 1), NaN, 'parseInt("3", 1) must return NaN');
assert.sameValue(parseInt("4", 1), NaN, 'parseInt("4", 1) must return NaN');
assert.sameValue(parseInt("5", 1), NaN, 'parseInt("5", 1) must return NaN');
assert.sameValue(parseInt("6", 1), NaN, 'parseInt("6", 1) must return NaN');
assert.sameValue(parseInt("7", 1), NaN, 'parseInt("7", 1) must return NaN');
assert.sameValue(parseInt("8", 1), NaN, 'parseInt("8", 1) must return NaN');
assert.sameValue(parseInt("9", 1), NaN, 'parseInt("9", 1) must return NaN');
assert.sameValue(parseInt("10", 1), NaN, 'parseInt("10", 1) must return NaN');
assert.sameValue(parseInt("11", 1), NaN, 'parseInt("11", 1) must return NaN');
