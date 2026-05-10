// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: AsyncGeneratorFunction.prototype property descriptor
esid: sec-asyncgeneratorfunction-prototype
info: |
  This property has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [async-iteration]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

verifyProperty(AsyncGeneratorFunction, "prototype", {
  enumerable: false,
  writable: false,
  configurable: false,
});
