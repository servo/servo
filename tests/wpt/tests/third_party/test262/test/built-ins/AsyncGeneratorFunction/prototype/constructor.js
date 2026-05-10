// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgeneratorfunction-prototype-constructor
description: >
  `constructor` property of the AsyncGeneratorFunction.prototype object
info: |
  The initial value of AsyncGeneratorFunction.prototype.constructor is the intrinsic
  object %AsyncGeneratorFunction%.

  This property has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [async-iteration]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

assert.sameValue(AsyncGeneratorFunction.prototype.constructor, AsyncGeneratorFunction);

verifyProperty(AsyncGeneratorFunction.prototype, "constructor", {
  enumerable: false,
  writable: false,
  configurable: true,
});
