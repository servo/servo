// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Number instances have no special properties beyond those
    inherited from the Number prototype object
es5id: 15.7.5_A1_T03
description: Checking property toLocaleString
---*/
assert.sameValue(
  (new Number()).hasOwnProperty("toLocaleString"),
  false,
  '(new Number()).hasOwnProperty("toLocaleString") must return false'
);

assert.sameValue(
  (new Number()).toLocaleString,
  Number.prototype.toLocaleString,
  'The value of (new Number()).toLocaleString is expected to equal the value of Number.prototype.toLocaleString'
);
