// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The String.prototype property has the attribute DontEnum
es5id: 15.5.3.1_A2
description: Checking if enumerating the String.prototype property fails
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#0
if (!(String.hasOwnProperty('prototype'))) {
  throw new Test262Error('#0: String.hasOwnProperty(\'prototype\') return true. Actual: ' + String.hasOwnProperty('prototype'));
}
//
//////////////////////////////////////////////////////////////////////////////


//////////////////////////////////////////////////////////////////////////////
// CHECK#1
if (String.propertyIsEnumerable('prototype')) {
  throw new Test262Error('#1: String.propertyIsEnumerable(\'prototype\') return false. Actual: ' + String.propertyIsEnumerable('prototype'));
}
//
//////////////////////////////////////////////////////////////////////////////


//////////////////////////////////////////////////////////////////////////////
// CHECK#2
var count = 0;

for (var p in String) {
  if (p === "prototype") count++;
}

if (count !== 0) {
  throw new Test262Error('#2: count=0; for (p in String){ if (p==="prototype") count++;}; count === 0. Actual: count ===' + count);
}
//
//////////////////////////////////////////////////////////////////////////////
