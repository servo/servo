// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object function is called with one argument value,
    and the value neither is null nor undefined, and is supplied, return ToObject(value)
es5id: 15.2.1.1_A2_T9
description: >
    Calling Object function with function argument value. The function
    is declared
---*/
function func() {
  return 1;
}

assert.sameValue(typeof func, 'function', 'The value of `typeof func` is expected to be "function"');

var n_obj = Object(func);

assert.sameValue(n_obj, func, 'The value of n_obj is expected to equal the value of func');
assert.sameValue(n_obj(), 1, 'n_obj() must return 1');
