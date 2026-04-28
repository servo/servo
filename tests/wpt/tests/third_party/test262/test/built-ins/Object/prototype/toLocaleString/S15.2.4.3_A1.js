// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: toLocaleString function returns the result of calling toString()
es5id: 15.2.4.3_A1
description: >
    Checking the type of Object.prototype.toLocaleString and the
    returned result
---*/
assert.sameValue(
  typeof Object.prototype.toLocaleString,
  "function",
  'The value of `typeof Object.prototype.toLocaleString` is expected to be "function"'
);

assert.sameValue(
  Object.prototype.toLocaleString(),
  Object.prototype.toString(),
  'Object.prototype.toLocaleString() must return the same value returned by Object.prototype.toString()'
);

assert.sameValue(
  {}.toLocaleString(),
  {}.toString(),
  '({}).toLocaleString() must return the same value returned by ({}).toString()'
);
