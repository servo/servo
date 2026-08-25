// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toUpperCase() return a string, but not a String object
es5id: 15.5.4.18_A2_T1
description: Checking returned result
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if ("Hello, WoRlD!".toUpperCase() !== "HELLO, WORLD!") {
  throw new Test262Error('#1: "Hello, WoRlD!".toUpperCase() === "HELLO, WORLD!". Actual: ' + ("Hello, WoRlD!".toUpperCase()));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if ("Hello, WoRlD!".toUpperCase() !== String("HELLO, WORLD!")) {
  throw new Test262Error('#2: "Hello, WoRlD!".toUpperCase() === String("HELLO, WORLD!"). Actual: ' + ("Hello, WoRlD!".toUpperCase()));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if ("Hello, WoRlD!".toUpperCase() === new String("HELLO, WORLD!")) {
  throw new Test262Error('#3: "Hello, WoRlD!".toUpperCase() !== new String("HELLO, WORLD!")');
}
//
//////////////////////////////////////////////////////////////////////////////
