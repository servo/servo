// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object(value) is called and the value is null, undefined or not supplied,
    create and return a new Object object if the object constructor had been called with the same arguments (15.2.2.1)
es5id: 15.2.1.1_A1_T1
description: Creating Object(null) and checking its properties
---*/

var __obj = Object(null);

var n__obj = new Object(null);

assert.sameValue(
  __obj.toString(),
  n__obj.toString(),
  '__obj.toString() must return the same value returned by n__obj.toString()'
);

assert.sameValue(
  __obj.constructor,
  n__obj.constructor,
  'The value of __obj.constructor is expected to equal the value of n__obj.constructor'
);

assert.sameValue(
  __obj.prototype,
  n__obj.prototype,
  'The value of __obj.prototype is expected to equal the value of n__obj.prototype'
);

assert.sameValue(
  __obj.toLocaleString(),
  n__obj.toLocaleString(),
  '__obj.toLocaleString() must return the same value returned by n__obj.toLocaleString()'
);

assert.sameValue(typeof __obj, typeof n__obj, 'The value of `typeof __obj` is expected to be typeof n__obj');
