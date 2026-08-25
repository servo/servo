// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.replace (searchValue, replaceValue)
es5id: 15.5.4.11_A1_T2
description: >
    Argument is function that return boolean, and instance is Boolean
    object
---*/

var __instance = new Boolean;

__instance.replace = String.prototype.replace;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.replace(function() {
    return false;
  }(), x) !== "undefined") {
  throw new Test262Error('#1: var x; __instance = new Boolean; __instance.replace = String.prototype.replace;  __instance.replace(function(){return false;}(),x) === "undefined". Actual: ' + __instance.replace(function() {
    return false;
  }(), x));
}
//
//////////////////////////////////////////////////////////////////////////////

var x;
