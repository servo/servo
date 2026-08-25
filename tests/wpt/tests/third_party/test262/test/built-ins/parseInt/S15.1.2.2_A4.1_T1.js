// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If R = 0 or R = undefined, then R = 10
esid: sec-parseint-string-radix
description: R = 0
---*/

assert.sameValue(parseInt("0", 0), parseInt("0", 10), 'parseInt("0", 0) must return the same value returned by parseInt("0", 10)');
assert.sameValue(parseInt("1", 0), parseInt("1", 10), 'parseInt("1", 0) must return the same value returned by parseInt("1", 10)');
assert.sameValue(parseInt("2", 0), parseInt("2", 10), 'parseInt("2", 0) must return the same value returned by parseInt("2", 10)');
assert.sameValue(parseInt("3", 0), parseInt("3", 10), 'parseInt("3", 0) must return the same value returned by parseInt("3", 10)');
assert.sameValue(parseInt("4", 0), parseInt("4", 10), 'parseInt("4", 0) must return the same value returned by parseInt("4", 10)');
assert.sameValue(parseInt("5", 0), parseInt("5", 10), 'parseInt("5", 0) must return the same value returned by parseInt("5", 10)');
assert.sameValue(parseInt("6", 0), parseInt("6", 10), 'parseInt("6", 0) must return the same value returned by parseInt("6", 10)');
assert.sameValue(parseInt("7", 0), parseInt("7", 10), 'parseInt("7", 0) must return the same value returned by parseInt("7", 10)');
assert.sameValue(parseInt("8", 0), parseInt("8", 10), 'parseInt("8", 0) must return the same value returned by parseInt("8", 10)');
assert.sameValue(parseInt("9", 0), parseInt("9", 10), 'parseInt("9", 0) must return the same value returned by parseInt("9", 10)');
assert.sameValue(parseInt("10", 0), parseInt("10", 10), 'parseInt("10", 0) must return the same value returned by parseInt("10", 10)');
assert.sameValue(parseInt("11", 0), parseInt("11", 10), 'parseInt("11", 0) must return the same value returned by parseInt("11", 10)');
assert.sameValue(parseInt("9999", 0), parseInt("9999", 10), 'parseInt("9999", 0) must return the same value returned by parseInt("9999", 10)');
