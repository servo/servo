// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split(separator, limit):
    i) can be transferred to other kinds of objects for use as a method.
    separator and limit can be any kinds of object since:
    ii) if separator is not RegExp ToString(separator) performs and
    iii) ToInteger(limit) performs
es5id: 15.5.4.14_A1_T14
description: >
    Arguments are objects, and instance is string.  First object have
    overrided toString function, that throw exception.  Second object
    have overrided valueOf function, that throw exception
---*/

var __obj = {
  toString: function() {
    throw "intostr";
  }
};
var __obj2 = {
  valueOf: function() {
    throw "intoint";
  }
};
var __instance = new Number(10001.10001);
Number.prototype.split = String.prototype.split;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var x = __instance.split(__obj, __obj2);
  Test262Error.thrower('#1: "var x = __instance.split(__obj, __obj2)" lead to throwing exception');
} catch (e) {
  assert.sameValue(e, "intoint", 'The value of `e` is "intoint"');
}
//
//////////////////////////////////////////////////////////////////////////////
