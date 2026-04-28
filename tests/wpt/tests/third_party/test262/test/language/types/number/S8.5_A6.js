// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: -Infinity expression has a type Number
es5id: 8.5_A6
description: Check type of -Infinity
---*/

var x=-Infinity;

///////////////////////////////////////////////////////
// CHECK#1
if (typeof(x) !== "number"){
  throw new Test262Error('#1: var x=-Infinity; typeof(x) === "number". Actual: ' + (typeof(x)));
}
//
//////////////////////////////////////////////////////////

///////////////////////////////////////////////////////
// CHECK#2
if (typeof(-Infinity) !== "number"){
  throw new Test262Error('#2: typeof(-Infinity) === "number". Actual: ' + (typeof(-Infinity)));
}
//
//////////////////////////////////////////////////////////
