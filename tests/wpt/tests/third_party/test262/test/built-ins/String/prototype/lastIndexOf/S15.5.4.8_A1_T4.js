// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.lastIndexOf(searchString, position)
es5id: 15.5.4.8_A1_T4
description: >
    Call lastIndexOf(searchString, position) function without
    arguments of string
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToString() evaluates to "" lastIndexOf() evaluates to lastIndexOf("",0)
if ("".lastIndexOf() !== -1) {
  throw new Test262Error('#1: "".lastIndexOf() === -1. Actual: ' + ("".lastIndexOf()));
}
//
//////////////////////////////////////////////////////////////////////////////
