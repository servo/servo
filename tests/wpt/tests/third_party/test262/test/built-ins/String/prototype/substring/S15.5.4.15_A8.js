// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The String.prototype.substring.length property has the attribute DontEnum
es5id: 15.5.4.15_A8
description: >
    Checking if enumerating the String.prototype.substring.length
    property fails
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#0
if (!(String.prototype.substring.hasOwnProperty('length'))) {
  throw new Test262Error('#0: String.prototype.substring.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.substring.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////


//////////////////////////////////////////////////////////////////////////////
// CHECK#1
if (String.prototype.substring.propertyIsEnumerable('length')) {
  throw new Test262Error('#1: String.prototype.substring.propertyIsEnumerable(\'length\') return false');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
// CHECK#2
var count = 0;

for (var p in String.prototype.substring) {
  if (p === "length") count++;
}

if (count !== 0) {
  throw new Test262Error('#2: count=0; for (p in String.prototype.substring){if (p==="length") count++;} count === 0. Actual: ' + count);
}
//
//////////////////////////////////////////////////////////////////////////////
