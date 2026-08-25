// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When Array is called as a function rather than as a constructor,
    it creates and initialises a new Array object
es5id: 15.4.1_A3.1_T1
description: Checking use typeof, instanceof
---*/

assert.sameValue(typeof Array(), "object", 'The value of `typeof Array()` is expected to be "object"');

assert.sameValue(
  Array() instanceof Array,
  true,
  'The result of evaluating (Array() instanceof Array) is expected to be true'
);
