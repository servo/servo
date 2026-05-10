// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-generatorfunction-prototype-object
description: Object extensibility
info: |
  The initial value of the [[Extensible]] internal slot of the
  GeneratorFunction prototype object is true.
features: [generators]
---*/

var GeneratorFunction = Object.getPrototypeOf(function*() {}).constructor;

assert(Object.isExtensible(GeneratorFunction.prototype));
