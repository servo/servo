// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.slice (start, end) can be applied to object instances
es5id: 15.5.4.13_A3_T4
description: >
    Checknig if applying String.prototype.slice to Function object
    instance passes
---*/

__FACTORY.prototype.toString = function() {
  return this.value + '';
};

var __instance = new __FACTORY(void 0);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.slice(0, 100) !== "undefined") {
  throw new Test262Error('#1: __instance.slice(0,100) === "undefined". Actual: ' + __instance.slice(0, 100));
}
//
//////////////////////////////////////////////////////////////////////////////

function __FACTORY(value) {
  this.value = value,
    this.slice = String.prototype.slice;
  //this.substring = String.prototype.substring;
}
