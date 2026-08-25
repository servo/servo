// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    [[HasInstance]] returns a boolean value indicating whether Value
    delegates behaviour to this object
es5id: 8.6.2_A4
description: >
    Check that the obj instance of Object, but not instance  of
    Function, String, Number, Array
---*/

var __obj={};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(__obj instanceof Object)) {
  throw new Test262Error('#1: var __obj={}; (__obj instanceof Object) === true. Actual: ' + ((__obj instanceof Object)));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__obj instanceof Function) {
  throw new Test262Error('#2: var __obj={}; (__obj instanceof Function) === false. Actual: ' + ((__obj instanceof Function)));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (__obj instanceof String) {
  throw new Test262Error('#3: var __obj={}; (__obj instanceof String) === false. Actual: ' + ((__obj instanceof String)));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if (__obj instanceof Number) {
  throw new Test262Error('#4: var __obj={}; (__obj instanceof Number) === false. Actual: ' + ((__obj instanceof Number)));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#5
if (__obj instanceof Array) {
  throw new Test262Error('#5: var __obj={}; (__obj instanceof Array) === false. Actual: ' + ((__obj instanceof Array)));
}
//
//////////////////////////////////////////////////////////////////////////////
