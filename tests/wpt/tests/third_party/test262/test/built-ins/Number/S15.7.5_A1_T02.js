// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Number instances have no special properties beyond those
    inherited from the Number prototype object
es5id: 15.7.5_A1_T02
description: Checking property toString
---*/
assert.sameValue(
  (new Number()).hasOwnProperty("toString"),
  false,
  '(new Number()).hasOwnProperty("toString") must return false'
);

assert.sameValue(
  (new Number()).toString,
  Number.prototype.toString,
  'The value of (new Number()).toString is expected to equal the value of Number.prototype.toString'
);
