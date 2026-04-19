// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Number instances have no special properties beyond those
    inherited from the Number prototype object
es5id: 15.7.5_A1_T01
description: Checking property constructor
---*/
assert.sameValue(
  (new Number()).hasOwnProperty("constructor"),
  false,
  '(new Number()).hasOwnProperty("constructor") must return false'
);

assert.sameValue(
  (new Number()).constructor,
  Number.prototype.constructor,
  'The value of (new Number()).constructor is expected to equal the value of Number.prototype.constructor'
);
