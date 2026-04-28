// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When String is called as a function rather than as a constructor, it
    performs a type conversion
es5id: 15.5.1.1_A1_T17
description: >
    Call String() with numbers that have more than 1 significant digit
    after point
---*/

var __str = String(1.2345);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof __str !== "string") {
  throw new Test262Error('#1: __str = String(1.2345); typeof __str === "string". Actual: typeof __str ===' + typeof __str);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__str !== "1.2345") {
  throw new Test262Error('#2: __str = String(1.2345); __str === "1.2345". Actual: __str ===' + __str);
}
//
//////////////////////////////////////////////////////////////////////////////

__str = String(1.234567890);

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (typeof __str !== "string") {
  throw new Test262Error('#3: __str = String(1.234567890); typeof __str === "string". Actual: typeof __str ===' + typeof __str);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if (__str !== "1.23456789") {
  throw new Test262Error('#4: __str = String(1.234567890); __str === "1.23456789". Actual: __str ===' + __str);
}
//
//////////////////////////////////////////////////////////////////////////////

__str = String(1.234500000000000000000000000);

//////////////////////////////////////////////////////////////////////////////
//CHECK#5
if (typeof __str !== "string") {
  throw new Test262Error('#5: __str = String(1.234500000000000000000000000); typeof __str === "string". Actual: typeof __str ===' + typeof __str);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#6
if (__str !== "1.2345") {
  throw new Test262Error('#6: __str = String(1.234500000000000000000000000); __str === "1.2345". Actual: __str ===' + __str);
}
//
//////////////////////////////////////////////////////////////////////////////
