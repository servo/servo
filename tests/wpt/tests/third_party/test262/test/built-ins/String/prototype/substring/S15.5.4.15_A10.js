// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The String.prototype.substring.length property has the attribute ReadOnly
es5id: 15.5.4.15_A10
description: >
    Checking if varying the String.prototype.substring.length property
    fails
includes: [propertyHelper.js]
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.substring.hasOwnProperty('length'))) {
  throw new Test262Error('#1: String.prototype.substring.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.substring.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

var __obj = String.prototype.substring.length;

verifyNotWritable(String.prototype.substring, "length", null, function() {
  return "shifted";
});

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.substring.length !== __obj) {
  throw new Test262Error('#2: __obj = String.prototype.substring.length; String.prototype.substring.length = function(){return "shifted";}; String.prototype.substring.length === __obj. Actual: ' + String.prototype.substring.length);
}
//
//////////////////////////////////////////////////////////////////////////////
