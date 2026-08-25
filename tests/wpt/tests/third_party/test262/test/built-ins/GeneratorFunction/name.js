// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-generatorfunction-constructor
description: Function "name" property
info: |
  The value of the name property of the GeneratorFunction is
  "GeneratorFunction".

  17 ECMAScript Standard Built-in Objects:
  Every built-in Function object, including constructors, that is not
  identified as an anonymous function has a name property whose value is a
  String.

  Unless otherwise specified, the name property of a built-in Function object,
  if it exists, has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [generators]
---*/

var GeneratorFunction = Object.getPrototypeOf(function*() {}).constructor;

verifyProperty(GeneratorFunction, "name", {
  value: "GeneratorFunction",
  writable: false,
  enumerable: false,
  configurable: true
});
