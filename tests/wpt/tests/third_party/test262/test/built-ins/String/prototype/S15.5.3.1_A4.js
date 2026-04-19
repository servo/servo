// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The String.prototype property has the attribute ReadOnly
es5id: 15.5.3.1_A4
description: Checking if varying the String.prototype property fails
includes: [propertyHelper.js]
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.hasOwnProperty('prototype'))) {
  throw new Test262Error('#1: String.hasOwnProperty(\'prototype\') return true. Actual: ' + String.hasOwnProperty('prototype'));
}
//
//////////////////////////////////////////////////////////////////////////////

var __obj = String.prototype;

verifyNotWritable(String, "prototype", null, function() {
  return "shifted";
});

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype !== __obj) {
  throw new Test262Error('#2: __obj = String.prototype; String.prototype = function(){return "shifted";}; String.prototype === __obj. Actual: String.prototype ===' + String.prototype);
}
//
//////////////////////////////////////////////////////////////////////////////
