// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The String.prototype.slice.length property has the attribute ReadOnly
es5id: 15.5.4.13_A10
description: >
    Checking if varying the String.prototype.slice.length property
    fails
includes: [propertyHelper.js]
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.slice.hasOwnProperty('length'))) {
  throw new Test262Error('#1: String.prototype.slice.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.slice.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

var __obj = String.prototype.slice.length;

verifyNotWritable(String.prototype.slice, "length", null, function() {
  return "shifted";
});

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.slice.length !== __obj) {
  throw new Test262Error('#2: __obj = String.prototype.slice.length; String.prototype.slice.length = function(){return "shifted";}; String.prototype.slice.length === __obj. Actual: ' + String.prototype.slice.length);
}
//
//////////////////////////////////////////////////////////////////////////////
