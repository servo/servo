// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.slice (start, end)
es5id: 15.5.4.13_A1_T12
description: >
    Arguments are objects, and instance is string.  First object have
    overrided valueOf function and toString function, that return
    exception.  Second object have overrided valueOf function, that
    return exception
---*/

var __obj = {
  valueOf: function() {
    return {};
  },
  toString: function() {
    throw "instart";
  }
};
var __obj2 = {
  valueOf: function() {
    throw "inend";
  }
};
var __str = new String("ABB\u0041BABAB");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var x = __str.slice(__obj, __obj2);
  throw new Test262Error('#1: "var x = __str.slice(__obj,__obj2)" lead to throwing exception');
} catch (e) {
  if (e !== "instart") {
    throw new Test262Error('#1.1: Exception === "instart". Actual: ' + e);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
