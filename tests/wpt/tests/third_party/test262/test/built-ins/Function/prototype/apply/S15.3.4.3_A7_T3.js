// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If argArray is either an array or an arguments object,
    the function is passed the (ToUint32(argArray.length)) arguments argArray[0], argArray[1],...,argArray[ToUint32(argArray.length)-1]
es5id: 15.3.4.3_A7_T3
description: argArray is (empty object, new Array("nine","inch","nails"))
---*/

var i = 0;

var p = {
  toString: function() {
    return "a" + (++i);
  }
};

var obj = {};

Function(p, "a2,a3", "this.shifted=a1;").apply(obj, new Array("nine", "inch", "nails"));

assert.sameValue(obj["shifted"], "nine", 'The value of obj["shifted"] is expected to be "nine"');

assert.sameValue(
  typeof this["shifted"],
  "undefined",
  'The value of `typeof this["shifted"]` is expected to be "undefined"'
);
