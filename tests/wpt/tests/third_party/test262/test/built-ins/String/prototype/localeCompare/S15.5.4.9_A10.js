// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The String.prototype.localeCompare.length property has the attribute
    ReadOnly
es5id: 15.5.4.9_A10
description: >
    Checking if varying the String.prototype.localeCompare.length
    property fails
includes: [propertyHelper.js]
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.localeCompare.hasOwnProperty('length'))) {
  throw new Test262Error('#1: String.prototype.localeCompare.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.localeCompare.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

var __obj = String.prototype.localeCompare.length;

verifyNotWritable(String.prototype.localeCompare, "length", null, function() {
  return "shifted";
});

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.localeCompare.length !== __obj) {
  throw new Test262Error('#2: __obj = String.prototype.localeCompare.length; String.prototype.localeCompare.length = function(){return "shifted";}; String.prototype.localeCompare.length === __obj. Actual: ' + String.prototype.localeCompare.length);
}
//
//////////////////////////////////////////////////////////////////////////////
