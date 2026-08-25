// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToInt32
esid: sec-parseint-string-radix
description: If radix is NaN, +0, -0, +Infinity, -Infinity, return radix = +0
---*/

assert.sameValue(parseInt("11", NaN), parseInt("11", 10), 'parseInt("11", NaN) must return the same value returned by parseInt("11", 10)');
assert.sameValue(parseInt("11", +0), parseInt("11", 10), 'parseInt("11", +0) must return the same value returned by parseInt("11", 10)');
assert.sameValue(parseInt("11", -0), parseInt("11", 10), 'parseInt("11", -0) must return the same value returned by parseInt("11", 10)');

assert.sameValue(
  parseInt("11", Number.POSITIVE_INFINITY),
  parseInt("11", 10),
  'parseInt("11", Number.POSITIVE_INFINITY) must return the same value returned by parseInt("11", 10)'
);

assert.sameValue(
  parseInt("11", Number.NEGATIVE_INFINITY),
  parseInt("11", 10),
  'parseInt("11", Number.NEGATIVE_INFINITY) must return the same value returned by parseInt("11", 10)'
);
