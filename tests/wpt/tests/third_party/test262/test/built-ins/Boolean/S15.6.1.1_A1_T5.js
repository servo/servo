// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Returns a boolean value (not a Boolean object) computed by
    ToBoolean(value)
esid: sec-terms-and-definitions-boolean-value
description: Used various assigning values to any variable as argument
---*/

var x;

assert.sameValue(typeof Boolean(x = 0), "boolean", 'The value of `typeof Boolean(x = 0)` is expected to be "boolean"');
assert.sameValue(Boolean(x = 0), false, 'Boolean(x = 0) must return false');
assert.sameValue(typeof Boolean(x = 1), "boolean", 'The value of `typeof Boolean(x = 1)` is expected to be "boolean"');
assert.sameValue(Boolean(x = 1), true, 'Boolean(x = 1) must return true');

assert.sameValue(
  typeof Boolean(x = false),
  "boolean",
  'The value of `typeof Boolean(x = false)` is expected to be "boolean"'
);

assert.sameValue(Boolean(x = false), false, 'Boolean(x = false) must return false');

assert.sameValue(
  typeof Boolean(x = true),
  "boolean",
  'The value of `typeof Boolean(x = true)` is expected to be "boolean"'
);

assert.sameValue(Boolean(x = true), true, 'Boolean(x = true) must return true');

assert.sameValue(
  typeof Boolean(x = null),
  "boolean",
  'The value of `typeof Boolean(x = null)` is expected to be "boolean"'
);

assert.sameValue(Boolean(x = null), false, 'Boolean(x = null) must return false');
