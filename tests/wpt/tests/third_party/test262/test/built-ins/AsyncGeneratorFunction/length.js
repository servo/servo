// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgeneratorfunction-length
description: >
  This is a data property with a value of 1. This property has the attributes
  { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [async-iteration]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

verifyProperty(AsyncGeneratorFunction, "length", {
  value: 1,
  enumerable: false,
  writable: false,
  configurable: true,
});
