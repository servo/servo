// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLocaleUpperCase()
es5id: 15.5.4.19_A1_T11
description: >
    Override toString function, toString throw exception, then call
    toLocaleUpperCase() function for this object
---*/

var __obj = {
  toString: function() {
    throw "intostr";
  }
}
__obj.toLocaleUpperCase = String.prototype.toLocaleUpperCase;
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var x = __obj.toLocaleUpperCase();
  throw new Test262Error('#1: "var x = __obj.toLocaleUpperCase()" lead to throwing exception');
} catch (e) {
  if (e !== "intostr") {
    throw new Test262Error('#1.1: Exception === "intostr". Actual: ' + e);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
