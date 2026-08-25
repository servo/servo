// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When String is called as a function rather than as a constructor, it
    performs a type conversion
es5id: 15.5.1.1_A1_T2
description: Call String(null)
---*/

var __str = String(null);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof __str !== "string") {
  throw new Test262Error('#1: __str = String(null); typeof __str === "string". Actual: typeof __str ===' + typeof __str);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__str !== "null") {
  throw new Test262Error('#2: __str = String(null); __str === "null". Actual: __str ===' + __str);
}
//
//////////////////////////////////////////////////////////////////////////////
