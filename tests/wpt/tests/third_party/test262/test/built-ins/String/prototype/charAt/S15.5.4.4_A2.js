// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When String.prototype.charAt(pos) calls if ToInteger(pos) less than 0 the
    empty string returns
es5id: 15.5.4.4_A2
description: Call charAt(pos) with negative pos
---*/

function __FACTORY() {};

__FACTORY.prototype.charAt = String.prototype.charAt;

var __instance = new __FACTORY;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.charAt(-1) !== "") {
  throw new Test262Error('#1: __instance.charAt(-1) === "". Actual: __instance.charAt(-1) ===' + __instance.charAt(-1));
}
//
//////////////////////////////////////////////////////////////////////////////
