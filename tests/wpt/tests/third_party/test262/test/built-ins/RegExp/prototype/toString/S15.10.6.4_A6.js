// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: RegExp.prototype.toString has not prototype property
es5id: 15.10.6.4_A6
description: Checking RegExp.prototype.toString.prototype
includes: [isConstructor.js]
features: [Reflect.construct]
---*/
assert.sameValue(
  RegExp.prototype.toString.prototype,
  undefined,
  'The value of RegExp.prototype.toString.prototype is expected to equal undefined'
);

assert.sameValue(
  isConstructor(RegExp.prototype.toString),
  false,
  'isConstructor(RegExp.prototype.toString) must return false'
);
