// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.charAt can't be used as constructor
es5id: 15.5.4.4_A7
description: Checking if creating the String.prototype.charAt object fails
---*/

var __FACTORY = String.prototype.charAt;

try {
  var __instance = new __FACTORY;
  throw new Test262Error('#1: __FACTORY = String.prototype.charAt; "__instance = new __FACTORY" lead to throwing exception');
} catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#1.2: undefined = 1 throw a TypeError. Actual: ' + (e));
  }
}
