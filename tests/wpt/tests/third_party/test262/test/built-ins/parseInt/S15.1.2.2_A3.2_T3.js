// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToInt32
esid: sec-parseint-string-radix
description: ToInt32 use modulo
---*/

assert.sameValue(
  parseInt("11", 4294967298),
  parseInt("11", 2),
  'parseInt("11", 4294967298) must return the same value returned by parseInt("11", 2)'
);
assert.sameValue(
  parseInt("11", 4294967296),
  parseInt("11", 10),
  'parseInt("11", 4294967296) must return the same value returned by parseInt("11", 10)'
);

assert.sameValue(parseInt("11", -2147483650), NaN, 'parseInt("11", -2147483650) must return NaN');

assert.sameValue(
  parseInt("11", -4294967294),
  parseInt("11", 2),
  'parseInt("11", -4294967294) must return the same value returned by parseInt("11", 2)'
);
