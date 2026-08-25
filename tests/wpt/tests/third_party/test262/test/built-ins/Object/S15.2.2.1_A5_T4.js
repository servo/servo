// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object constructor is called with one argument value and
    the type of value is Number, return ToObject(number)
es5id: 15.2.2.1_A5_T4
description: Argument value is numeric expression
---*/

var n_obj = new Object(2 * ([].length + {
  q: 1
}["q"]));

assert.sameValue(n_obj.constructor, Number, 'The value of n_obj.constructor is expected to equal the value of Number');
assert.sameValue(typeof n_obj, 'object', 'The value of `typeof n_obj` is expected to be "object"');
assert(n_obj == 2, 'The result of evaluating (n_obj == 2) is expected to be true');
assert.notSameValue(n_obj, 2, 'The value of n_obj is not 2');
