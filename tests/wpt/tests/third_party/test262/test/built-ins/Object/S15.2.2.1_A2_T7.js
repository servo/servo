// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object constructor is called with one argument value and
    the value is a native ECMAScript object, do not create a new object but simply return value
es5id: 15.2.2.1_A2_T7
description: The value is a function declaration
---*/
assert.sameValue(typeof func, 'undefined', 'The value of `typeof func` is expected to be "undefined"');

var n_obj = new Object(function func() {
  return 1;
});

assert.sameValue(
  n_obj.constructor,
  Function,
  'The value of n_obj.constructor is expected to equal the value of Function'
);

assert.sameValue(n_obj(), 1, 'n_obj() must return 1');
assert.sameValue(typeof func, 'undefined', 'The value of `typeof func` is expected to be "undefined"');
