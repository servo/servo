// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The call method takes one or more arguments, thisArg and (optionally) arg1, arg2 etc, and performs
    a function call using the [[Call]] property of the object
es5id: 15.3.4.4_A6_T3
description: >
    Argunemts of call function is (empty object, new
    Array("nine","inch","nails"))
---*/

var i = 0;

var p = {
  toString: function() {
    return "a" + (++i);
  }
};

var obj = {};

Function(p, "a2,a3", "this.shifted=a1;").call(obj, new Array("nine", "inch", "nails"));

assert.sameValue(obj["shifted"].length, 3);

if ((obj["shifted"][0] !== "nine") || (obj["shifted"][1] !== "inch") || (obj["shifted"][2] !== "nails")) {
  throw new Test262Error('#2: The call method takes one or more arguments, thisArg and (optionally) arg1, arg2 etc, and performs a function call using the [[Call]] property of the object');
}

if (typeof this["shifted"] !== "undefined") {
  throw new Test262Error('#3: The call method takes one or more arguments, thisArg and (optionally) arg1, arg2 etc, and performs a function call using the [[Call]] property of the object');
}
