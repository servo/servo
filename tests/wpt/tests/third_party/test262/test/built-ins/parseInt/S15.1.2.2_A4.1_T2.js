// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If R = 0 or R = undefined, then R = 10
esid: sec-parseint-string-radix
description: R = undefined
---*/

assert.sameValue(parseInt("0"), parseInt("0", 10), 'parseInt("0") must return the same value returned by parseInt("0", 10)');
assert.sameValue(parseInt("1"), parseInt("1", 10), 'parseInt("1") must return the same value returned by parseInt("1", 10)');
assert.sameValue(parseInt("2"), parseInt("2", 10), 'parseInt("2") must return the same value returned by parseInt("2", 10)');
assert.sameValue(parseInt("3"), parseInt("3", 10), 'parseInt("3") must return the same value returned by parseInt("3", 10)');
assert.sameValue(parseInt("4"), parseInt("4", 10), 'parseInt("4") must return the same value returned by parseInt("4", 10)');
assert.sameValue(parseInt("5"), parseInt("5", 10), 'parseInt("5") must return the same value returned by parseInt("5", 10)');
assert.sameValue(parseInt("6"), parseInt("6", 10), 'parseInt("6") must return the same value returned by parseInt("6", 10)');
assert.sameValue(parseInt("7"), parseInt("7", 10), 'parseInt("7") must return the same value returned by parseInt("7", 10)');
assert.sameValue(parseInt("8"), parseInt("8", 10), 'parseInt("8") must return the same value returned by parseInt("8", 10)');
assert.sameValue(parseInt("9"), parseInt("9", 10), 'parseInt("9") must return the same value returned by parseInt("9", 10)');
assert.sameValue(parseInt("10"), parseInt("10", 10), 'parseInt("10") must return the same value returned by parseInt("10", 10)');
assert.sameValue(parseInt("11"), parseInt("11", 10), 'parseInt("11") must return the same value returned by parseInt("11", 10)');
assert.sameValue(parseInt("9999"), parseInt("9999", 10), 'parseInt("9999") must return the same value returned by parseInt("9999", 10)');
