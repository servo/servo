// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    when String.prototype.indexOf(searchString, position) is called first Call ToString, giving it the this value as its argument.
    Then Call ToString(searchString) and Call ToNumber(position)
es5id: 15.5.4.7_A4_T1
description: Override toString and valueOf functions, valueOf throw exception
---*/

var __obj = {
  toString: function() {
    return "\u0041B";
  }
}
var __obj2 = {
  valueOf: function() {
    throw "intointeger";
  }
}
var __str = "ABB\u0041BABAB";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var x = __str.indexOf(__obj, __obj2);
  throw new Test262Error('#1: "var x = __str.indexOf(__obj, __obj2)" lead to throwing exception');
} catch (e) {
  if (e !== "intointeger") {
    throw new Test262Error('#1.1: Exception === "intointeger". Actual: ' + e);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
