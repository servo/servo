// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toUpperCase()
es5id: 15.5.4.18_A1_T4
description: >
    Call toUpperCase() function without arguments of string and from
    empty string
---*/

var __lowerCase = "".toUpperCase();

var __expected = "";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__lowerCase.length !== __expected.length) {
  throw new Test262Error('#1: __lowerCase = "".toUpperCase(); __expected = ""; __lowerCase.length === __expected.length. Actual: ' + __lowerCase.length);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__lowerCase.index !== __expected.index) {
  throw new Test262Error('#2: __lowerCase = "".toUpperCase(); __expected = ""; __lowerCase.index === __expected.index. Actual: ' + __lowerCase.index);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (__lowerCase.input !== __expected.input) {
  throw new Test262Error('#3: __lowerCase = "".toUpperCase(); __expected = ""; __lowerCase.input === __expected.input. Actual: ' + __lowerCase.input);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if (__lowerCase[0] !== __expected[0]) {
  throw new Test262Error('#4: __lowerCase = "".toUpperCase(); __expected = ""; __lowerCase[0]===' + __expected[0] + '. Actual: ' + __lowerCase[0]);
}
//
//////////////////////////////////////////////////////////////////////////////
