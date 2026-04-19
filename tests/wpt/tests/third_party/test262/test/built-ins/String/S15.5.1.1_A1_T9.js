// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When String is called as a function rather than as a constructor, it
    performs a type conversion
es5id: 15.5.1.1_A1_T9
description: Call String(this)
---*/

var toString = function() {
  return "__THIS__";
};

var __str = String(this);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof __str !== "string") {
  throw new Test262Error('#1: __str = String(this); typeof __str === "string". Actual: typeof __str ===' + typeof __str);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__str !== "__THIS__") {
  throw new Test262Error('#2: toString=function(){return "__THIS__";}; __str = String(this); __str === "__THIS__". Actual: __str ===' + __str);
}
//
//////////////////////////////////////////////////////////////////////////////
