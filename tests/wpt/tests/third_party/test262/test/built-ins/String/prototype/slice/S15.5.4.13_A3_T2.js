// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.slice (start, end) can be applied to object instances
es5id: 15.5.4.13_A3_T2
description: >
    Apply String.prototype.slice to Object instance, use other value
    for start and end values
---*/

var __instance = new Object();

__instance.slice = String.prototype.slice;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.slice(8, __instance.toString().length) !== "Object]") {
  throw new Test262Error('#1: __instance = new Object(); __instance.slice = String.prototype.slice; __instance.slice(8,__instance.toString().length) === "Object]". Actual: ' + __instance.slice(8, __instance.toString().length));
}
//
//////////////////////////////////////////////////////////////////////////////
