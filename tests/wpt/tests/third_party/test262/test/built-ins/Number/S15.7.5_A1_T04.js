// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Number instances have no special properties beyond those
    inherited from the Number prototype object
es5id: 15.7.5_A1_T04
description: Checking property valueOf
---*/
assert.sameValue(
  (new Number()).hasOwnProperty("valueOf"),
  false,
  '(new Number()).hasOwnProperty("valueOf") must return false'
);

assert.sameValue(
  (new Number()).valueOf,
  Number.prototype.valueOf,
  'The value of (new Number()).valueOf is expected to equal the value of Number.prototype.valueOf'
);
