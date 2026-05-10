// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.match (regexp)
es5id: 15.5.4.10_A1_T11
description: >
    Override toString function, toString throw exception, then call
    match (regexp) function with this object as argument
---*/

var __obj = {
  toString: function() {
    throw "intostr";
  }
}
var __str = "ABB\u0041BABAB";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var x = __str.match(__obj);
  throw new Test262Error('#1: "var x = __str.match(__obj)" lead to throwing exception');
} catch (e) {
  if (e !== "intostr") {
    throw new Test262Error('#1.1: Exception === "intostr". Actual: ' + e);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
