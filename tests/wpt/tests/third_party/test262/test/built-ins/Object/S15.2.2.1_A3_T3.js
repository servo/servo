// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object constructor is called with one argument value and
    the type of value is String, return ToObject(string)
es5id: 15.2.2.1_A3_T3
description: Argument value is sum of empty string and number
---*/

var n_obj = new Object("" + 1);

assert.sameValue(n_obj.constructor, String, 'The value of n_obj.constructor is expected to equal the value of String');
assert.sameValue(typeof n_obj, 'object', 'The value of `typeof n_obj` is expected to be "object"');
assert(n_obj == "1", 'The result of evaluating (n_obj == "1") is expected to be true');
assert.notSameValue(n_obj, "1", 'The value of n_obj is not "1"');
