// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-456
description: >
    ES5 Attributes - fail to update [[Configurable]] attribute of
    accessor property ([[Get]] is undefined, [[Set]] is undefined,
    [[Enumerable]] is false, [[Configurable]] is false) to different
    value
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: undefined,
  enumerable: false,
  configurable: false
});
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");

try {
  Object.defineProperty(obj, "prop", {
    configurable: true
  });

  throw new Test262Error("Expected TypeError");
} catch (e) {
  assert(e instanceof TypeError);

  assert.sameValue(desc1.configurable, false);
}

verifyProperty(obj, "prop", {
  configurable: false,
});
