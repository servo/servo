// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The call method takes one or more arguments, thisArg and (optionally) arg1, arg2 etc, and performs
    a function call using the [[Call]] property of the object
es5id: 15.3.4.4_A6_T9
description: >
    Argunemts of call function is (empty object, "", arguments,2),
    inside function declaration used
---*/

function FACTORY() {
  var obj = {};
  Function("a1,a2,a3", "this.shifted=a1+a2.length+a3;").call(obj, "", arguments, 2);
  return obj;
}

var obj = new FACTORY("", 1, 2, void 0);

assert.sameValue(
  typeof this["shifted"],
  "undefined",
  'The value of `typeof this["shifted"]` is expected to be "undefined"'
);

assert.sameValue(obj.shifted, "42", 'The value of obj.shifted is expected to be "42"');
