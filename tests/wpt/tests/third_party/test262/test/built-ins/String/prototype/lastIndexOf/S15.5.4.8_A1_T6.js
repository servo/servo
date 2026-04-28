// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.lastIndexOf(searchString, position)
es5id: 15.5.4.8_A1_T6
description: >
    Call lastIndexOf(searchString, position) function with x argument
    of new String object, where x is undefined variable
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToString(undefined) evaluates to "" lastIndexOf(undefined) evaluates to lastIndexOf("",0)
if (new String("undefined").lastIndexOf(x) !== 0) {
  throw new Test262Error('#1: var x; new String("undefined").lastIndexOf(x) === 0. Actual: ' + new String("undefined").lastIndexOf(x));
}
//
//////////////////////////////////////////////////////////////////////////////

var x;
