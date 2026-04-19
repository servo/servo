// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the [[HasProperty]] method of O is called with property name P and if O has not a property with name P
    then If the [[Prototype]] of O is null, return false or call the [[HasProperty]] method of [[Prototype]] with property name P
es5id: 8.12.6_A2_T1
description: Try find not existent property of any Object
---*/

var __obj={};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!("valueOf" in __obj)) {
  throw new Test262Error('#1: var __obj={}; "valueOf" in __obj');
}
//
//////////////////////////////////////////////////////////////////////////////
