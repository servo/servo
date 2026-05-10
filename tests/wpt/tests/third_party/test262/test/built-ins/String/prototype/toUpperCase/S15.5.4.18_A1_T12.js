// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toUpperCase()
es5id: 15.5.4.18_A1_T12
description: >
    Override toString and valueOf functions, valueOf throw exception,
    then call toUpperCase() function for this object
---*/

var __obj = {
  toString: function() {
    return {};
  },
  valueOf: function() {
    throw "intostr";
  }
}
__obj.toUpperCase = String.prototype.toUpperCase;
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var x = __obj.toUpperCase();
  throw new Test262Error('#1: "var x = __obj.toUpperCase()" lead to throwing exception');
} catch (e) {
  if (e !== "intostr") {
    throw new Test262Error('#1.1: Exception === "intostr". Actual: ' + e);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
