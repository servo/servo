// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The initial value of Boolean.prototype is the Boolean
    prototype object
esid: sec-boolean.prototype
description: Checking Boolean.prototype property
---*/

assert.sameValue(
  typeof Boolean.prototype,
  "object",
  'The value of `typeof Boolean.prototype` is expected to be "object"'
);

assert(Boolean.prototype == false, 'The value of Boolean.prototype is expected to be false');

delete Boolean.prototype.toString;

assert.sameValue(
  Boolean.prototype.toString(),
  "[object Boolean]",
  'Boolean.prototype.toString() must return "[object Boolean]"'
);
