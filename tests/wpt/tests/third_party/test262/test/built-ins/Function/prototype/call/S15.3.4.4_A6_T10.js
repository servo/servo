// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The call method takes one or more arguments, thisArg and (optionally) arg1, arg2 etc, and performs
    a function call using the [[Call]] property of the object
es5id: 15.3.4.4_A6_T10
description: >
    Argunemts of call function is (empty object, "", arguments,2),
    inside function call without declaration used
---*/

var obj = {};

(function() {
  Function("a1,a2,a3", "this.shifted=a1.length+a2+a3;").call(obj, arguments, "", "2");
})("", 4, 2, "a");

assert.sameValue(obj["shifted"], "42", 'The value of obj["shifted"] is expected to be "42"');

assert.sameValue(
  typeof this["shifted"],
  "undefined",
  'The value of `typeof this["shifted"]` is expected to be "undefined"'
);
