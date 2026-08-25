// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-boolean.prototype.tostring
info: |
    toString: If this boolean value is true, then the string "true"
    is returned, otherwise, this boolean value must be false, and the string
    "false" is returned
es5id: 15.6.4.2_A1_T2
description: with some argument
---*/
assert.sameValue(Boolean.prototype.toString(true), "false", 'Boolean.prototype.toString(true) must return "false"');
assert.sameValue((new Boolean()).toString(true), "false", '(new Boolean()).toString(true) must return "false"');

assert.sameValue(
  (new Boolean(false)).toString(true),
  "false",
  '(new Boolean(false)).toString(true) must return "false"'
);

assert.sameValue(
  (new Boolean(true)).toString(false),
  "true",
  '(new Boolean(true)).toString(false) must return "true"'
);

assert.sameValue((new Boolean(1)).toString(false), "true", '(new Boolean(1)).toString(false) must return "true"');
assert.sameValue((new Boolean(0)).toString(true), "false", '(new Boolean(0)).toString(true) must return "false"');

assert.sameValue(
  (new Boolean(new Object())).toString(false),
  "true",
  '(new Boolean(new Object())).toString(false) must return "true"'
);
