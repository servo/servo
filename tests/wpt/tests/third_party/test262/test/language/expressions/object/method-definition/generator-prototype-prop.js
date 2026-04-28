// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Generator functions declared as methods define a `prototype` property.
es6id: 14.4.13
includes: [propertyHelper.js]
features: [generators]
---*/

var GeneratorPrototype = Object.getPrototypeOf(function* () {}).prototype;
var method = { *method() {} }.method;

assert.sameValue(
  Object.getPrototypeOf(method.prototype),
  GeneratorPrototype
);

verifyProperty(method, "prototype", {
  writable: true,
  enumerable: false,
  configurable: false,
});
