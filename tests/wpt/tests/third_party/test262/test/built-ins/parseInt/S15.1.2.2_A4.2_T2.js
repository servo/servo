// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If R < 2 or R > 36, then return NaN
esid: sec-parseint-string-radix
description: R = 37
---*/

assert.sameValue(parseInt("0", 37), NaN, 'parseInt("0", 37) must return NaN');
assert.sameValue(parseInt("1", 37), NaN, 'parseInt("1", 37) must return NaN');
assert.sameValue(parseInt("2", 37), NaN, 'parseInt("2", 37) must return NaN');
assert.sameValue(parseInt("3", 37), NaN, 'parseInt("3", 37) must return NaN');
assert.sameValue(parseInt("4", 37), NaN, 'parseInt("4", 37) must return NaN');
assert.sameValue(parseInt("5", 37), NaN, 'parseInt("5", 37) must return NaN');
assert.sameValue(parseInt("6", 37), NaN, 'parseInt("6", 37) must return NaN');
assert.sameValue(parseInt("7", 37), NaN, 'parseInt("7", 37) must return NaN');
assert.sameValue(parseInt("8", 37), NaN, 'parseInt("8", 37) must return NaN');
assert.sameValue(parseInt("9", 37), NaN, 'parseInt("9", 37) must return NaN');
assert.sameValue(parseInt("10", 37), NaN, 'parseInt("10", 37) must return NaN');
assert.sameValue(parseInt("11", 37), NaN, 'parseInt("11", 37) must return NaN');
