// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: length property has the attributes {ReadOnly}
es5id: 15.5.5.1_A4_T2
description: Checking if varying the length property of String fails
includes: [propertyHelper.js]
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
if (__str__instance.length !== 8) {
  throw new Test262Error('#2: var __str__instance = new String("globglob"); __str__instance.length === 8. Actual: __str__instance.length ===' + __str__instance.length);
}
//
//////////////////////////////////////////////////////////////////////////////

verifyNotWritable(__str__instance, "length", null, -1);

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (__str__instance.length !== 8) {
  throw new Test262Error('#3: var __str__instance = new String("globglob"); __str__instance.length=-1; __str__instance.length === 8(after redefine length property). Actual: __str__instance.length ===' + __str__instance.length);
}
//
//////////////////////////////////////////////////////////////////////////////
