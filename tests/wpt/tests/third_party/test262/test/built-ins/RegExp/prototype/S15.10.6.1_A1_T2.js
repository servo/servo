// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The initial value of RegExp.prototype.constructor is the built-in RegExp
    constructor
es5id: 15.10.6.1_A1_T2
description: >
    Compare instance.constructor !== RegExp, where instance is new
    RegExp.prototype.constructor
---*/

var __FACTORY = RegExp.prototype.constructor;

var __instance = new __FACTORY;

assert.sameValue(
  __instance instanceof RegExp,
  true,
  'The result of evaluating (__instance instanceof RegExp) is expected to be true'
);

assert.sameValue(
  __instance.constructor,
  RegExp,
  'The value of __instance.constructor is expected to equal the value of RegExp'
);
