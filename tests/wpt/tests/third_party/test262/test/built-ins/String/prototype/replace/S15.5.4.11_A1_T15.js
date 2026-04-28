// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.replace (searchValue, replaceValue)
es5id: 15.5.4.11_A1_T15
description: Instance is Object, searchValue is regular expression
---*/

var __obj = {
  toString: function() {
    return /77/
  }
};

var __instance = new Object(1100.00777001);

Object.prototype.replace = String.prototype.replace;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var x = __instance.replace(__obj, 1) === "1100.0017001";
  throw new Test262Error('#1.0: x = __instance.replace(__obj, 1) === "1100.0017001" lead to throwing exception');
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error('#1.1: Exception is instance of TypeError. Actual: ' + e);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
