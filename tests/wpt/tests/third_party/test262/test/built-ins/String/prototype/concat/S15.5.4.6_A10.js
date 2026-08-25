// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The String.prototype.concat.length property has the attribute ReadOnly
es5id: 15.5.4.6_A10
description: >
    Checking if varying the String.prototype.concat.length property
    fails
includes: [propertyHelper.js]
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.concat.hasOwnProperty('length'))) {
  throw new Test262Error('#1: String.prototype.concat.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.concat.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

var __obj = String.prototype.concat.length;

verifyNotWritable(String.prototype.concat, "length", null, function() {
  return "shifted";
});

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.concat.length !== __obj) {
  throw new Test262Error('#2: __obj = String.prototype.concat.length; String.prototype.concat.length = function(){return "shifted";}; String.prototype.concat.length === __obj. Actual: ' + String.prototype.concat.length);
}
//
//////////////////////////////////////////////////////////////////////////////
