// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Number instances have no special properties beyond those
    inherited from the Number prototype object
es5id: 15.7.5_A1_T06
description: Checking property toExponential
---*/
assert.sameValue(
  (new Number()).hasOwnProperty("toExponential"),
  false,
  '(new Number()).hasOwnProperty("toExponential") must return false'
);

assert.sameValue(
  (new Number()).toExponential,
  Number.prototype.toExponential,
  'The value of (new Number()).toExponential is expected to equal the value of Number.prototype.toExponential'
);
