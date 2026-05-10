// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "[[hasProperty]] is sensitive to property existence but [[Get]] is not"
es5id: 8.12.6_A3
description: >
    Use [[hasProperty]] and [[Get]] for existent and not existent
    properties
---*/

var __obj={}; __obj.hole=undefined;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__obj.hole !== undefined) {
  throw new Test262Error('#1: var __obj={}; __obj.hole=undefined; __obj.hole === undefined. Actual: ' + (__obj.hole));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__obj.notexist !== undefined) {
  throw new Test262Error('#2: var __obj={}; __obj.hole=undefined; __obj.notexist === undefined. Actual: ' + (__obj.notexist));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (!("hole" in __obj)) {
  throw new Test262Error('#3: var __obj={}; __obj.hole=undefined; "hole" in __obj');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if (("notexist" in __obj)) {
  throw new Test262Error('#4: var __obj={}; __obj.hole=undefined; "notexist" in __obj');
}
//
//////////////////////////////////////////////////////////////////////////////
