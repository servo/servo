// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.replace (searchValue, replaceValue)
es5id: 15.5.4.11_A1_T13
description: >
    Call replace (searchValue, replaceValue) function with objects
    arguments of string.  First objects have overrided toString and
    valueOf functions.  Second objects have overrided toString
    function, that throw exception
---*/

var __obj = {
  toString: function() {
    return {};
  },
  valueOf: function() {
    return 1;
  }
};
var __obj2 = {
  toString: function() {
    throw "inreplaceValue";
  }
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var x = "ABB\u0041BABAB\u0031BBAA".replace(__obj, __obj2);
  throw new Test262Error('#1: var x = "ABB\\u0041BABAB\\u0031BBAA".replace(__obj,__obj2) lead to throwing exception');
} catch (e) {
  if (e !== "inreplaceValue") {
    throw new Test262Error('#1.1: Exception === "inreplaceValue". Actual: ' + e);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
