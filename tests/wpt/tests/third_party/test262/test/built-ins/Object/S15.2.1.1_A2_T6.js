// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object function is called with one argument value,
    and the value neither is null nor undefined, and is supplied, return ToObject(value)
es5id: 15.2.1.1_A2_T6
description: Calling Object function with Infinity argument value
---*/

var num = Infinity;

assert.sameValue(typeof num, 'number', 'The value of `typeof num` is expected to be "number"');

var obj = Object(num);

assert.sameValue(obj.constructor, Number, 'The value of obj.constructor is expected to equal the value of Number');
assert.sameValue(typeof obj, "object", 'The value of `typeof obj` is expected to be "object"');
assert(obj == Infinity, 'The result of evaluating (obj == Infinity) is expected to be true');
assert.notSameValue(obj, Infinity, 'The value of obj is expected to not equal ``Infinity``');
