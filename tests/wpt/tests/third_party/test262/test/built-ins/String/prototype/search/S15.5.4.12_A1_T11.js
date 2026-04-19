// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.search (regexp)
es5id: 15.5.4.12_A1_T11
description: >
    Argument is object, and instance is string.  Object with overrided
    toString function, that throw exception
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
  var x = __str.search(__obj);
  throw new Test262Error('#1: var x = __str.search(__obj) lead to throwing exception');
} catch (e) {
  if (e !== "intostr") {
    throw new Test262Error('#1.1: Exception === "intostr". Actual: ' + e);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
