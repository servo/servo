// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split(separator, limit):
    i) can be transferred to other kinds of objects for use as a method.
    separator and limit can be any kinds of object since:
    ii) if separator is not RegExp ToString(separator) performs and
    iii) ToInteger(limit) performs
es5id: 15.5.4.14_A1_T12
description: >
    Arguments are objects, and instance is string.  First object have
    overrided toString function.  Second object have overrided valueOf
    function and toString function, that throw exception
---*/

var __obj = {
  toString: function() {
    return "\u0041B";
  }
}
var __obj2 = {
  valueOf: function() {
    return {};
  },
  toString: function() {
    throw "intointeger";
  }
}
var __str = new String("ABB\u0041BABAB");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var x = __str.split(__obj, __obj2);
  Test262Error.thrower('#1: "var x = __str.split(__obj, __obj2)" lead to throwing exception');
} catch (e) {
  assert.sameValue(e, "intointeger", 'The value of `e` is "intointeger"');
}
//
//////////////////////////////////////////////////////////////////////////////
