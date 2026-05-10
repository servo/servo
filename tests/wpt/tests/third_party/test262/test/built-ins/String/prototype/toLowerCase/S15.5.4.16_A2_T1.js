// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLowerCase() return a string, but not a String object
es5id: 15.5.4.16_A2_T1
description: Checking returned result
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if ("Hello, WoRlD!".toLowerCase() !== "hello, world!") {
  throw new Test262Error('#1: "Hello, WoRlD!".toLowerCase() === "hello, world!". Actual: ' + ("Hello, WoRlD!".toLowerCase()));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if ("Hello, WoRlD!".toLowerCase() !== String("hello, world!")) {
  throw new Test262Error('#2: "Hello, WoRlD!".toLowerCase() === String("hello, world!"). Actual: ' + ("Hello, WoRlD!".toLowerCase()));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if ("Hello, WoRlD!".toLowerCase() === new String("hello, world!")) {
  throw new Test262Error('#3: "Hello, WoRlD!".toLowerCase() !== new String("hello, world!")');
}
//
//////////////////////////////////////////////////////////////////////////////
