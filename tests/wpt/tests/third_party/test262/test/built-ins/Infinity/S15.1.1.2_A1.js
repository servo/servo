// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The initial value of Infinity is Number.POSITIVE_INFINITY
es5id: 15.1.1.2_A1
description: Use typeof, isNaN, isFinite
---*/
assert.sameValue(typeof(Infinity), "number", 'The value of `typeof(Infinity)` is expected to be "number"');
assert.sameValue(isFinite(Infinity), false, 'isFinite(Infinity) must return false');
assert.sameValue(isNaN(Infinity), false, 'isNaN(Infinity) must return false');

assert.sameValue(
  Infinity,
  Number.POSITIVE_INFINITY,
  'The value of Infinity is expected to equal the value of Number.POSITIVE_INFINITY'
);
