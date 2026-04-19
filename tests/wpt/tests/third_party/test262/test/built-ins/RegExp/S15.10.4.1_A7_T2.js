// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Prototype]] property of the newly constructed object is set to the
    original RegExp prototype object, the one that is the initial value of
    RegExp.prototype
es5id: 15.10.4.1_A7_T2
description: Checking [[Prototype]] property of the newly constructed object
---*/

var __re = new RegExp();

assert.sameValue(
  RegExp.prototype.isPrototypeOf(__re),
  true,
  'RegExp.prototype.isPrototypeOf(new RegExp()) must return true'
);
