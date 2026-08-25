// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
info: |
    Returns a boolean value (not a Boolean object) computed by
    ToBoolean(value)
esid: sec-terms-and-definitions-boolean-value
description: Used various string values as argument
---*/

assert.sameValue(typeof Boolean("0"), "boolean", 'The value of `typeof Boolean("0")` is expected to be "boolean"');
assert.sameValue(Boolean("0"), true, 'Boolean("0") must return true');
assert.sameValue(typeof Boolean("-1"), "boolean", 'The value of `typeof Boolean("-1")` is expected to be "boolean"');
assert.sameValue(Boolean("-1"), true, 'Boolean("-1") must return true');
assert.sameValue(typeof Boolean("1"), "boolean", 'The value of `typeof Boolean("1")` is expected to be "boolean"');
assert.sameValue(Boolean("1"), true, 'Boolean("1") must return true');

assert.sameValue(
  typeof Boolean("false"),
  "boolean",
  'The value of `typeof Boolean("false")` is expected to be "boolean"'
);

assert.sameValue(Boolean("false"), true, 'Boolean("false") must return true');

assert.sameValue(
  typeof Boolean("true"),
  "boolean",
  'The value of `typeof Boolean("true")` is expected to be "boolean"'
);

assert.sameValue(Boolean("true"), true, 'Boolean("true") must return true');
