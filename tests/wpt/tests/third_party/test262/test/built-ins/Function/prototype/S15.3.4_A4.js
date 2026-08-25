// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The Function prototype object does not have a valueOf property of its
    own. however, it inherits the valueOf property from the Object prototype
    Object
es5id: 15.3.4_A4
description: Checking valueOf property at Function.prototype
---*/
assert.sameValue(
  Function.prototype.hasOwnProperty("valueOf"),
  false,
  'Function.prototype.hasOwnProperty("valueOf") must return false'
);

assert.notSameValue(
  typeof Function.prototype.valueOf,
  "undefined",
  'The value of typeof Function.prototype.valueOf is not "undefined"'
);

assert.sameValue(
  Function.prototype.valueOf,
  Object.prototype.valueOf,
  'The value of Function.prototype.valueOf is expected to equal the value of Object.prototype.valueOf'
);
