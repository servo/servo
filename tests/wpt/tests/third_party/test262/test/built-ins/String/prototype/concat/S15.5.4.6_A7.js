// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.concat can't be used as constructor
es5id: 15.5.4.6_A7
description: Checking if creating the String.prototype.concat object fails
---*/

var __FACTORY = String.prototype.concat;

try {
  var __instance = new __FACTORY;
  throw new Test262Error('#1: __FACTORY = String.prototype.concat; "__instance = new __FACTORY" lead throwing exception');
} catch (e) {
  if (e instanceof Test262Error) throw e;
}
