// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-return
description: >
  AsyncGenerator.prototype.return.length is 1.
info: |
  AsyncGenerator.prototype.return ( value )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, has a length
    property whose value is an integer. Unless otherwise specified, this
    value is equal to the largest number of named arguments shown in the
    subclause headings for the function description, including optional
    parameters. However, rest parameters shown using the form “...name”
    are not included in the default argument count.

    Unless otherwise specified, the length property of a built-in Function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [async-iteration]
---*/

async function* g() {}
var AsyncGeneratorPrototype = Object.getPrototypeOf(g).prototype;

verifyProperty(AsyncGeneratorPrototype.return, "length", {
  value: 1,
  enumerable: false,
  writable: false,
  configurable: true,
});

