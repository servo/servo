// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-boolean.prototype.tostring
info: |
    toString: If this boolean value is true, then the string "true"
    is returned, otherwise, this boolean value must be false, and the string
    "false" is returned
es5id: 15.6.4.2_A1_T1
description: no arguments
---*/
assert.sameValue(Boolean.prototype.toString(), "false", 'Boolean.prototype.toString() must return "false"');
assert.sameValue((new Boolean()).toString(), "false", '(new Boolean()).toString() must return "false"');
assert.sameValue((new Boolean(false)).toString(), "false", '(new Boolean(false)).toString() must return "false"');
assert.sameValue((new Boolean(true)).toString(), "true", '(new Boolean(true)).toString() must return "true"');
assert.sameValue((new Boolean(1)).toString(), "true", '(new Boolean(1)).toString() must return "true"');
assert.sameValue((new Boolean(0)).toString(), "false", '(new Boolean(0)).toString() must return "false"');

assert.sameValue(
  (new Boolean(new Object())).toString(),
  "true",
  '(new Boolean(new Object())).toString() must return "true"'
);
