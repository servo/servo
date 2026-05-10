// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the [[HasProperty]] method of O is called with property name P and
    if O has a property with name P, return true
es5id: 8.12.6_A1
description: Try find existent property of any Object
---*/

var __obj={fooProp:"fooooooo"};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!("fooProp" in __obj)) {
  throw new Test262Error('#1: var __obj={fooProp:"fooooooo"}; "fooProp" in __obj');
}
//
//////////////////////////////////////////////////////////////////////////////
