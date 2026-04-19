// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.substring (start, end) can be applied to non String object instance and
    returns a string value(not object)
es5id: 15.5.4.15_A3_T2
description: >
    Apply String.prototype.substring to Array instance. Start is 9,
    end is -Infinity
---*/

var __instance = new Array(1, 2, 3, 4, 5);
__instance.substring = String.prototype.substring;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.substring(9, -Infinity) !== "1,2,3,4,5") {
  throw new Test262Error('#1: __instance = new Array(1,2,3,4,5); __instance.substring = String.prototype.substring; __instance.substring(9,-Infinity) === "1,2,3,4,5". Actual: ' + __instance.substring(9, -Infinity));
}
//
//////////////////////////////////////////////////////////////////////////////
