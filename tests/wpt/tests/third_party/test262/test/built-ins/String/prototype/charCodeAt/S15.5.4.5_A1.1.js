// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.charCodeAt() can accept many arguments
es5id: 15.5.4.5_A1.1
description: Checking by using eval
---*/

function __FACTORY() {
  this.toString = function() {
    return "wizard";
  };
};

__FACTORY.prototype.charCodeAt = String.prototype.charCodeAt;

var __instance = new __FACTORY;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.charCodeAt(eval("1"), true, null, {}) !== 0x69) {
  throw new Test262Error('#1: __instance.charCodeAt(eval("1"),true,null,{})=== 0x69. Actual: __instance.charCodeAt(eval("1"),true,null,{})===' + __instance.charCodeAt(eval("1"), true, null, {}));
}
//
//////////////////////////////////////////////////////////////////////////////
