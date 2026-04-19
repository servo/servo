// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToNumber
esid: sec-parseint-string-radix
description: Checking for Boolean object
---*/

assert.sameValue(
  parseInt("11", new Boolean(false)),
  parseInt("11", false),
  'parseInt("11", new Boolean(false)) must return the same value returned by parseInt("11", false)'
);

//CHECK#2
assert.sameValue(parseInt("11", new Boolean(true)), NaN, 'parseInt("11", new Boolean(true)) must return NaN');
