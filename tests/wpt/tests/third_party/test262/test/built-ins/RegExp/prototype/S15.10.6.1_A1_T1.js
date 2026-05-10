// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The initial value of RegExp.prototype.constructor is the built-in RegExp
    constructor
es5id: 15.10.6.1_A1_T1
description: Compare RegExp.prototype.constructor with RegExp
---*/
assert.sameValue(
  RegExp.prototype.constructor,
  RegExp,
  'The value of RegExp.prototype.constructor is expected to equal the value of RegExp'
);
