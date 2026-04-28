// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.lastIndexOf(searchString, position)
es5id: 15.5.4.8_A1_T7
description: >
    Call lastIndexOf(searchString, position) function with undefined
    argument of string object
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToString(undefined) evaluates to "undefined" lastIndexOf(undefined) evaluates to lastIndexOf("undefined",0)
if (String("undefined").lastIndexOf(undefined) !== 0) {
  throw new Test262Error('#1: String("undefined").lastIndexOf(undefined) === 0. Actual: ' + String("undefined").lastIndexOf(undefined));
}
//
//////////////////////////////////////////////////////////////////////////////
