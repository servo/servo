// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If argArray is either an array or an arguments object,
    the function is passed the (ToUint32(argArray.length)) arguments argArray[0], argArray[1],...,argArray[ToUint32(argArray.length)-1]
es5id: 15.3.4.3_A7_T4
description: >
    argArray is (empty object, ( function(){return arguments;})
    ("a","b","c"))
---*/

var i = 0;

var p = {
  toString: function() {
    return "a" + (++i);
  }
};

var obj = {};

new Function(p, p, p, "this.shifted=a3;").apply(obj, (function() {
  return arguments;
})("a", "b", "c"));

assert.sameValue(obj["shifted"], "c", 'The value of obj["shifted"] is expected to be "c"');

assert.sameValue(
  typeof this["shifted"],
  "undefined",
  'The value of `typeof this["shifted"]` is expected to be "undefined"'
);
