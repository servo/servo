// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When String is called as a function rather than as a constructor, it
    performs a type conversion
es5id: 15.5.1.1_A1_T5
description: Call String(x), where x is undefined variable
---*/

var __str = String(x);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof __str !== "string") {
  throw new Test262Error('#1: var x; __str = String(x); typeof __str === "string". Actual: typeof __str ===' + typeof __str);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__str !== "undefined") {
  throw new Test262Error('#2: var x; __str = String(x); __str === "undefined". Actual: __str ===' + __str);
}
//
//////////////////////////////////////////////////////////////////////////////

var x;
