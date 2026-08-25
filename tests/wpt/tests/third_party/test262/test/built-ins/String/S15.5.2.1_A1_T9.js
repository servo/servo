// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When "String" is called as part of a new expression, it is a constructor: it initialises the newly created object and
    The [[Value]] property of the newly constructed object is set to ToString(value), or to the empty string if value is not supplied
es5id: 15.5.2.1_A1_T9
description: >
    Creating string object with "new String(function(){return
    [1,2,3]}())"
---*/

var __str = new String(function() {
  return [1, 2, 3]
}());

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof __str !== "object") {
  throw new Test262Error('#1: __str = new String(function(){return [1,2,3]}()); typeof __str === "object". Actual: typeof __str ===' + typeof __str);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#1.5
if (__str.constructor !== String) {
  throw new Test262Error('#1.5: __str = new String(function(){return [1,2,3]}()); __str.constructor === String. Actual: __str.constructor ===' + __str.constructor);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__str != "1,2,3") {
  throw new Test262Error('#2: __str = new String(function(){return [1,2,3]}()); __str =="1,2,3". Actual: __str ==' + __str);
}
//
//////////////////////////////////////////////////////////////////////////////
