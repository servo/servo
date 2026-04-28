// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The initial value of Object.prototype.constructor is the built-in Object
    constructor
es5id: 15.2.4.1_A1_T2
description: >
    Creating "new Object.prototype.constructor" and checking its
    properties
---*/

var constr = Object.prototype.constructor;

var obj = new constr;

assert.notSameValue(obj, undefined, 'The value of obj is expected to not equal ``undefined``');
assert.sameValue(obj.constructor, Object, 'The value of obj.constructor is expected to equal the value of Object');

assert(
  !!Object.prototype.isPrototypeOf(obj),
  'The value of !!Object.prototype.isPrototypeOf(obj) is expected to be true'
);

var to_string_result = '[object ' + 'Object' + ']';
assert.sameValue(obj.toString(), to_string_result, 'obj.toString() returns to_string_result');
assert.sameValue(obj.valueOf().toString(), to_string_result, 'obj.valueOf().toString() returns to_string_result');
