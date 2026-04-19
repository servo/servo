// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object constructor is called with one argument value and
    the type of value is Boolean, return ToObject(boolean)
es5id: 15.2.2.1_A4_T1
description: Argument value is "true"
---*/

var bool = true;

assert.sameValue(typeof bool, 'boolean', 'The value of `typeof bool` is expected to be "boolean"');

var n_obj = new Object(bool);

assert.sameValue(
  n_obj.constructor,
  Boolean,
  'The value of n_obj.constructor is expected to equal the value of Boolean'
);

assert.sameValue(typeof n_obj, 'object', 'The value of `typeof n_obj` is expected to be "object"');
assert(n_obj == bool, 'The result of evaluating (n_obj == bool) is expected to be true');
assert.notSameValue(n_obj, bool, 'The value of n_obj is expected to not equal the value of `bool`');
