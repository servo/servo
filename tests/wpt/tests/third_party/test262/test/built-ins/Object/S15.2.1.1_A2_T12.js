// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object function is called with one argument value,
    and the value neither is null nor undefined, and is supplied, return ToObject(value)
es5id: 15.2.1.1_A2_T12
description: Calling Object function with numeric expression as argument value
---*/

var obj = Object(1.1 * ([].length + {
  q: 1
}["q"]));

assert.sameValue(typeof obj, "object", 'The value of `typeof obj` is expected to be "object"');
assert.sameValue(obj.constructor, Number, 'The value of obj.constructor is expected to equal the value of Number');
assert(obj == 1.1, 'The result of evaluating (obj == 1.1) is expected to be true');
assert.notSameValue(obj, 1.1, 'The value of obj is not 1.1');
