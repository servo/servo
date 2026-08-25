// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.slice (start, end) can be applied to object instances
es5id: 15.5.4.13_A3_T3
description: >
    Apply String.prototype.slice to Object instance, and call
    instance.slice(...).slice(...)
---*/

var __instance = {
  toString: function() {
    return "function(){}";
  }
};

__instance.slice = String.prototype.slice;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.slice(-Infinity, 8).slice(1, Infinity) !== "unction") {
  throw new Test262Error('#1: __instance = function(){}; __instance.slice = String.prototype.slice; __instance.slice(-Infinity,8).slice(1,Infinity) === "unction". Actual: ' + __instance.slice(-Infinity, 8).slice(1, Infinity));
}
//
//////////////////////////////////////////////////////////////////////////////
