// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object constructor is called with no arguments the following steps are taken:
    (The argument value was not supplied or its type was Null or Undefined.)
    i)	Create a new native ECMAScript object.
    ii) 	The [[Prototype]] property of the newly constructed object is set to the Object prototype object.
    iii) 	The [[Class]] property of the newly constructed object is set to "Object".
    iv) 	The newly constructed object has no [[Value]] property.
    v) 	Return the newly created native object
es5id: 15.2.2.1_A1_T3
description: Creating new Object(null) and checking its properties
---*/

var obj = new Object(null);

assert.notSameValue(obj, undefined, 'The value of obj is expected to not equal ``undefined``');
assert.sameValue(obj.constructor, Object, 'The value of obj.constructor is expected to equal the value of Object');

assert(
  !!Object.prototype.isPrototypeOf(obj),
  'The value of !!Object.prototype.isPrototypeOf(obj) is expected to be true'
);

var to_string_result = '[object ' + 'Object' + ']';
assert.sameValue(obj.toString(), to_string_result, 'obj.toString() returns to_string_result');

assert.sameValue(
  obj.valueOf().toString(),
  to_string_result.toString(),
  'obj.valueOf().toString() must return the same value returned by to_string_result.toString()'
);
