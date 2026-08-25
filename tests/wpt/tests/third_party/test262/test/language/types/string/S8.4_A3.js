// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String type has a length property
es5id: 8.4_A3
description: Try read length property of string variable
---*/

var __str = "ABCDEFGH";
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__str.length !== 8) {
  throw new Test262Error('#1: var __str = "ABCDEFGH"; __str.length === 8. Actual: ' + (__str.length));
}
//
//////////////////////////////////////////////////////////////////////////////
