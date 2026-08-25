// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-generatorfunction-constructor
description: Object extensibility
info: |
  The value of the [[Extensible]] internal slot of the GeneratorFunction
  constructor is true.
features: [generators]
---*/

var GeneratorFunction = Object.getPrototypeOf(function*() {}).constructor;

assert(Object.isExtensible(GeneratorFunction));
