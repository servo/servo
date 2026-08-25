// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Object function is called with one argument value,
    and the value neither is null nor undefined, and is supplied, return ToObject(value)
es5id: 15.2.1.1_A2_T4
description: Calling Object function with object argument value
---*/

var obj = {
  flag: true
};

assert.sameValue(typeof(obj), 'object', 'The value of `typeof(obj)` is expected to be "object"');

var n_obj = Object(obj);

assert.sameValue(n_obj, obj, 'The value of n_obj is expected to equal the value of obj');
assert(!!n_obj['flag'], 'The value of !!n_obj["flag"] is expected to be true');
