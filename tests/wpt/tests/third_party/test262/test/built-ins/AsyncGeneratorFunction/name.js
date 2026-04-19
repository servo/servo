// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-asyncgeneratorfunction
description: Function "name" property
info: |
  The value of the name property of the AsyncGeneratorFunction
  is "AsyncGeneratorFunction".

  17 ECMAScript Standard Built-in Objects:
  Every built-in Function object, including constructors, that is not
  identified as an anonymous function has a name property whose value is a
  String.

  Unless otherwise specified, the name property of a built-in Function object,
  if it exists, has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [async-iteration]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

verifyProperty(AsyncGeneratorFunction, "name", {
  value: "AsyncGeneratorFunction",
  enumerable: false,
  writable: false,
  configurable: true,
});
