// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.toLocaleUpperCase() return a string, but not a String
    object
es5id: 15.5.4.19_A2_T1
description: Checking returned result
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if ("Hello, WoRlD!".toLocaleUpperCase() !== "HELLO, WORLD!") {
  throw new Test262Error('#1: "Hello, WoRlD!".toLocaleUpperCase() === "HELLO, WORLD!". Actual: ' + ("Hello, WoRlD!".toLocaleUpperCase()));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if ("Hello, WoRlD!".toLocaleUpperCase() !== String("HELLO, WORLD!")) {
  throw new Test262Error('#2: "Hello, WoRlD!".toLocaleUpperCase() === String("HELLO, WORLD!"). Actual: ' + ("Hello, WoRlD!".toLocaleUpperCase()));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if ("Hello, WoRlD!".toLocaleUpperCase() === new String("HELLO, WORLD!")) {
  throw new Test262Error('#3: "Hello, WoRlD!".toLocaleUpperCase() !== new String("HELLO, WORLD!"');
}
//
//////////////////////////////////////////////////////////////////////////////
