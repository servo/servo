// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.slice (start, end)
es5id: 15.5.4.13_A1_T2
description: >
    Arguments are function call and x, and instance is Boolean. x is
    undefined variable
---*/

var __instance = new Boolean;

__instance.slice = String.prototype.slice;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.slice(function() {
    return true;
  }(), x) !== "alse") {
  throw new Test262Error('#1: var x; __instance = new Boolean; __instance.slice = String.prototype.slice;  __instance.slice(function(){return true;}(),x) === "alse". Actual: ' + __instance.slice(function() {
    return true;
  }(), x));
}
//
//////////////////////////////////////////////////////////////////////////////

var x;
