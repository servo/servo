// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToNumber
esid: sec-parseint-string-radix
description: Checking for string primitive
---*/

assert.sameValue(parseInt("11", "2"), parseInt("11", 2), 'parseInt("11", "2") must return the same value returned by parseInt("11", 2)');
assert.sameValue(parseInt("11", "0"), parseInt("11", 10), 'parseInt("11", "0") must return the same value returned by parseInt("11", 10)');
assert.sameValue(parseInt("11", ""), parseInt("11", 10), 'parseInt("11", "") must return the same value returned by parseInt("11", 10)');
