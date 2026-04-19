// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.replace (searchValue, replaceValue)
es5id: 15.5.4.11_A1_T11
description: >
    Call replace (searchValue, replaceValue) function with objects
    arguments of string object. Objects have overrided toString
    function, that throw exception
---*/

var __obj = {
  toString: function() {
    throw "insearchValue";
  }
};
var __obj2 = {
  toString: function() {
    throw "inreplaceValue";
  }
};
var __str = "ABB\u0041BABAB";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var x = __str.replace(__obj, __obj2);
  throw new Test262Error('#1: "var x = __str.replace(__obj,__obj2)" lead to throwing exception');
} catch (e) {
  if (e !== "insearchValue") {
    throw new Test262Error('#1.1: Exception === "insearchValue". Actual: ' + e);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
