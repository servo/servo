// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Returns a boolean value (not a Boolean object) computed by
    ToBoolean(value)
esid: sec-terms-and-definitions-boolean-value
description: >
    Used values 1, new String("1"), new Object(1) and called without
    argument
---*/
assert.sameValue(typeof Boolean(), "boolean", 'The value of `typeof Boolean()` is expected to be "boolean"');
assert.sameValue(typeof Boolean(1), "boolean", 'The value of `typeof Boolean(1)` is expected to be "boolean"');

assert.sameValue(
  typeof Boolean(new String("1")),
  "boolean",
  'The value of `typeof Boolean(new String("1"))` is expected to be "boolean"'
);

assert.sameValue(
  typeof Boolean(new Object(1)),
  "boolean",
  'The value of `typeof Boolean(new Object(1))` is expected to be "boolean"'
);
