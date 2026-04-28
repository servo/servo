// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object constructor is called with one argument value and
    the type of value is String, return ToObject(string)
es5id: 15.2.2.1_A3_T2
description: Argument value is an empty string
---*/

var str = '';

assert.sameValue(typeof str, 'string', 'The value of `typeof str` is expected to be "string"');

var n_obj = new Object(str);

assert.sameValue(n_obj.constructor, String, 'The value of n_obj.constructor is expected to equal the value of String');
assert.sameValue(typeof n_obj, 'object', 'The value of `typeof n_obj` is expected to be "object"');
assert(n_obj == str, 'The result of evaluating (n_obj == str) is expected to be true');
assert.notSameValue(n_obj, str, 'The value of n_obj is expected to not equal the value of `str`');
