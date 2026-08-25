// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-tostringtag
description: >
    `Symbol.toStringTag` property descriptor
info: |
    The initial value of the @@toStringTag property is the String value
    "AsyncGenerator".

    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: true }.

includes: [propertyHelper.js]
features: [async-iteration, Symbol.toStringTag]
---*/

var AsyncGeneratorPrototype = Object.getPrototypeOf(
  Object.getPrototypeOf(async function*() {}())
);

verifyProperty(AsyncGeneratorPrototype, Symbol.toStringTag, {
  value: 'AsyncGenerator',
  enumerable: false,
  writable: false,
  configurable: true,
});
