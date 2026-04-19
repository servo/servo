// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: length property has the attributes {ReadOnly}
es5id: 15.5.5.1_A4_T1
description: Checking if varying the length property of String fails
flags: [noStrict]
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

__str__instance.length = -1;

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (__str__instance.length !== 8) {
  throw new Test262Error('#3: var __str__instance = new String("globglob"); __str__instance.length=-1; __str__instance.length === 8(after redefine length property). Actual: __str__instance.length ===' + __str__instance.length);
}
//
//////////////////////////////////////////////////////////////////////////////

with(__str__instance)
length = 0;

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if (__str__instance.length !== 8) {
  throw new Test262Error('#4: var __str__instance = new String("globglob"); with(__str__instance) length = 0; __str__instance.length === 8(after redefine length property with using "with"). Actual: __str__instance.length ===' + __str__instance.length);
}
//
//////////////////////////////////////////////////////////////////////////////

__str__instance.length++;

//////////////////////////////////////////////////////////////////////////////
//CHECK#5
if (__str__instance.length !== 8) {
  throw new Test262Error('#5: var __str__instance = new String("globglob"); __str__instance.length++; __str__instance.length === 8(after redefine length property with using "++"). Actual: __str__instance.length ===' + __str__instance.length);
}
//
//////////////////////////////////////////////////////////////////////////////
