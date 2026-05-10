// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToNumber
esid: sec-parseint-string-radix
description: Checking for boolean primitive
---*/

assert.sameValue(parseInt("11", false), parseInt("11", 10), 'parseInt("11", false) must return the same value returned by parseInt("11", 10)');

//CHECK#2
assert.sameValue(parseInt("11", true), NaN, 'parseInt("11", true) must return NaN');
