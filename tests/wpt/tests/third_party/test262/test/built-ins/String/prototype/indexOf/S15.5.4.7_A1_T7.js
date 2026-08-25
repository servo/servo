// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.indexOf(searchString, position)
es5id: 15.5.4.7_A1_T7
description: >
    Call indexOf(searchString, position) function with undefined
    argument of string object
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToString(undefined) evaluates to "undefined" indexOf(undefined) evaluates to indexOf("undefined",0)
if (String("undefined").indexOf(undefined) !== 0) {
  throw new Test262Error('#1: String("undefined").indexOf(undefined) === 0. Actual: ' + String("undefined").indexOf(undefined));
}
//
//////////////////////////////////////////////////////////////////////////////
