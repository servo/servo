// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Returns a boolean value (not a Boolean object) computed by
    ToBoolean(value)
esid: sec-terms-and-definitions-boolean-value
description: Used various undefined values and null as argument
---*/

assert.sameValue(
  typeof Boolean(undefined),
  "boolean",
  'The value of `typeof Boolean(undefined)` is expected to be "boolean"'
);

assert.sameValue(Boolean(undefined), false, 'Boolean(undefined) must return false');

assert.sameValue(
  typeof Boolean(void 0),
  "boolean",
  'The value of `typeof Boolean(void 0)` is expected to be "boolean"'
);

assert.sameValue(Boolean(void 0), false, 'Boolean(void 0) must return false');

assert.sameValue(
  typeof Boolean(function() {}()),
  "boolean",
  'The value of `typeof Boolean(function() {}())` is expected to be "boolean"'
);

assert.sameValue(Boolean(function() {}()), false, 'Boolean(function() {}()) must return false');
assert.sameValue(typeof Boolean(null), "boolean", 'The value of `typeof Boolean(null)` is expected to be "boolean"');
assert.sameValue(Boolean(null), false, 'Boolean(null) must return false');
assert.sameValue(typeof Boolean(x), "boolean", 'The value of `typeof Boolean(x)` is expected to be "boolean"');
assert.sameValue(Boolean(x), false, 'Boolean() must return false');
var x;
