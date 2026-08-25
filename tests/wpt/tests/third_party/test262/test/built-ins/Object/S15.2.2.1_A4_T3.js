// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object constructor is called with one argument value and
    the type of value is Boolean, return ToObject(boolean)
es5id: 15.2.2.1_A4_T3
description: Argument value is boolean expression
---*/

var n_obj = new Object((1 === 1) && !(false));

assert.sameValue(
  n_obj.constructor,
  Boolean,
  'The value of n_obj.constructor is expected to equal the value of Boolean'
);

assert.sameValue(typeof n_obj, 'object', 'The value of `typeof n_obj` is expected to be "object"');
assert(n_obj == true, 'The result of evaluating (n_obj == true) is expected to be true');
assert.notSameValue(n_obj, true, 'The value of n_obj is not true');
