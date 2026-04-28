// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The String.prototype.concat.length property has the attribute DontEnum
es5id: 15.5.4.6_A8
description: >
    Checking if enumerating the String.prototype.concat.length
    property fails
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#0
if (!(String.prototype.concat.hasOwnProperty('length'))) {
  throw new Test262Error('#0: String.prototype.concat.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.concat.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////


//////////////////////////////////////////////////////////////////////////////
// CHECK#1
if (String.prototype.concat.propertyIsEnumerable('length')) {
  throw new Test262Error('#1: String.prototype.concat.propertyIsEnumerable(\'length\') return false. Actual: ' + String.prototype.concat.propertyIsEnumerable('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
// CHECK#2
var count = 0;

for (var p in String.prototype.concat) {
  if (p === "length") count++;
}

if (count !== 0) {
  throw new Test262Error('#2: count = 0; for (p in String.prototype.concat){ if (p==="length") count++;}; count === 0. Actual: ' + count);
}
//
//////////////////////////////////////////////////////////////////////////////
