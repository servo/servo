// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString
esid: sec-parseint-string-radix
description: Checking for boolean primitive
---*/

assert.sameValue(parseInt(true), NaN, 'parseInt(true) must return NaN');
assert.sameValue(parseInt(false), NaN, 'parseInt(false) must return NaN');
