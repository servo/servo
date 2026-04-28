// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The String.prototype.lastIndexOf.length property has the attribute
    ReadOnly
es5id: 15.5.4.8_A10
description: >
    Checking if varying the String.prototype.lastIndexOf.length
    property fails
includes: [propertyHelper.js]
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.lastIndexOf.hasOwnProperty('length'))) {
  throw new Test262Error('#1: String.prototype.lastIndexOf.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.lastIndexOf.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

var __obj = String.prototype.lastIndexOf.length;

verifyNotWritable(String.prototype.lastIndexOf, "length", null, function() {
  return "shifted";
});

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.lastIndexOf.length !== __obj) {
  throw new Test262Error('#2: __obj = String.prototype.lastIndexOf.length; String.prototype.lastIndexOf.length = function(){return "shifted";}; String.prototype.lastIndexOf.length === __obj. Actual: ' + String.prototype.lastIndexOf.length);
}
//
//////////////////////////////////////////////////////////////////////////////
