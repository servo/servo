// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-constructor
description: >
    The GeneratorPrototype intrinsic's constructor.
info: |
    AsyncGenerator.prototype.constructor

    The initial value of AsyncGenerator.prototype.constructor is the
    intrinsic object %AsyncGenerator%.

    This property has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.

includes: [propertyHelper.js]
features: [async-iteration]
---*/

async function* g() {}
var AsyncGenerator = Object.getPrototypeOf(g);
var AsyncGeneratorPrototype = AsyncGenerator.prototype;

verifyProperty(AsyncGeneratorPrototype, 'constructor', {
  value: AsyncGenerator,
  enumerable: false,
  writable: false,
  configurable: true,
});
