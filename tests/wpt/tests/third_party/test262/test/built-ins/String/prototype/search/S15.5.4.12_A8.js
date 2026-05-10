// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The String.prototype.search.length property has the attribute DontEnum
es5id: 15.5.4.12_A8
description: >
    Checking if enumerating the String.prototype.search.length
    property fails
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#0
if (!(String.prototype.search.hasOwnProperty('length'))) {
  throw new Test262Error('#0: String.prototype.search.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.search.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////


//////////////////////////////////////////////////////////////////////////////
// CHECK#1
if (String.prototype.search.propertyIsEnumerable('length')) {
  throw new Test262Error('#1: String.prototype.search.propertyIsEnumerable(\'length\') return false');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
// CHECK#2
var count = 0;

for (var p in String.prototype.search) {
  if (p === "length") count++;
}

if (count !== 0) {
  throw new Test262Error('#2: count=0; for (p in String.prototype.search){if (p==="length") count++;}; count === 0. Actual: ' + count);
}
//
//////////////////////////////////////////////////////////////////////////////
