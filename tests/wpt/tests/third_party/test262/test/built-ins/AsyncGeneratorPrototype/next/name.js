// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-next
description: >
  Generator.prototype.next.name is "next".
info: |
  Generator.prototype.next ( value )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [async-iteration]
---*/

async function* g() {}
var AsyncGeneratorPrototype = Object.getPrototypeOf(g).prototype;

verifyProperty(AsyncGeneratorPrototype.next, "name", {
  value: "next",
  enumerable: false,
  writable: false,
  configurable: true,
});
