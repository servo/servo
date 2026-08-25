// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Global object properties have attributes { DontEnum }
es5id: 10.2.3_A2.1_T1
description: Global execution context - Value Properties
---*/

//CHECK#1
for (var x in this) {
  if (x === 'NaN') {
    throw new Test262Error("#1: 'NaN' have attribute DontEnum");
  } else if (x === 'Infinity') {
    throw new Test262Error("#1: 'Infinity' have attribute DontEnum");
  } else if (x === 'undefined') {
    throw new Test262Error("#1: 'undefined' have attribute DontEnum");
  }
}
