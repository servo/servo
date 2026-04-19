// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: length property has the attributes {DontEnum}
es5id: 15.5.5.1_A2
description: Checking if enumerating the length property of String fails
---*/

var __str__instance = new String("globglob");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(__str__instance.hasOwnProperty("length"))) {
  throw new Test262Error('#1: var __str__instance = new String("globglob"); __str__instance.hasOwnProperty("length") return true. Actual: ' + __str__instance.hasOwnProperty("length"));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
for (var prop in __str__instance) {
  if (prop === "length") {
    throw new Test262Error('#2: length property has the attributes {DontEnum}');
  }
}
//
//////////////////////////////////////////////////////////////////////////////
