// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToInt32
esid: sec-parseint-string-radix
description: ToInt32 use floor
---*/

assert.sameValue(parseInt("11", 2.1), parseInt("11", 2), 'parseInt("11", 2.1) must return the same value returned by parseInt("11", 2)');
assert.sameValue(parseInt("11", 2.5), parseInt("11", 2), 'parseInt("11", 2.5) must return the same value returned by parseInt("11", 2)');
assert.sameValue(parseInt("11", 2.9), parseInt("11", 2), 'parseInt("11", 2.9) must return the same value returned by parseInt("11", 2)');

assert.sameValue(
  parseInt("11", 2.000000000001),
  parseInt("11", 2),
  'parseInt("11", 2.000000000001) must return the same value returned by parseInt("11", 2)'
);

assert.sameValue(
  parseInt("11", 2.999999999999),
  parseInt("11", 2),
  'parseInt("11", 2.999999999999) must return the same value returned by parseInt("11", 2)'
);
