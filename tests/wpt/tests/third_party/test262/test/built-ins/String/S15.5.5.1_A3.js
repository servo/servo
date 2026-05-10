// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: length property has the attributes {DontDelete}
es5id: 15.5.5.1_A3
description: Checking if deleting the length property of String fails
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

verifyNotConfigurable(__str__instance, "length");

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
try {
  if (delete __str__instance.length === true) {
    throw new Test262Error('#2: var __str__instance = new String("globglob"); delete __str__instance.length !== true');
  }
} catch (e) {
  if (e instanceof Test262Error) throw e;
  assert(e instanceof TypeError);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (!(__str__instance.hasOwnProperty("length"))) {
  throw new Test262Error('#3: var __str__instance = new String("globglob"); delete __str__instance.length; __str__instance.hasOwnProperty("length") return true. Actual: ' + __str__instance.hasOwnProperty("length"));
}
//
//////////////////////////////////////////////////////////////////////////////
