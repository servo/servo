// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Returns a boolean value (not a Boolean object) computed by
    ToBoolean(value)
esid: sec-terms-and-definitions-boolean-value
description: Used various number values as argument
---*/
assert.sameValue(typeof Boolean(0), "boolean", 'The value of `typeof Boolean(0)` is expected to be "boolean"');
assert.sameValue(Boolean(0), false, 'Boolean(0) must return false');
assert.sameValue(typeof Boolean(-1), "boolean", 'The value of `typeof Boolean(-1)` is expected to be "boolean"');
assert.sameValue(Boolean(-1), true, 'Boolean(-1) must return true');

assert.sameValue(
  typeof Boolean(-Infinity),
  "boolean",
  'The value of `typeof Boolean(-Infinity)` is expected to be "boolean"'
);

assert.sameValue(Boolean(-Infinity), true, 'Boolean(-Infinity) must return true');
assert.sameValue(typeof Boolean(NaN), "boolean", 'The value of `typeof Boolean(NaN)` is expected to be "boolean"');
assert.sameValue(Boolean(NaN), false, 'Boolean(NaN) must return false');
