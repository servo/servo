// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%arrayiteratorprototype%.next
description: >
  %ArrayIteratorPrototype%.next.name is "next".
info: |
  %ArrayIteratorPrototype%.next( )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Symbol.iterator]
---*/

var ArrayIteratorProto = Object.getPrototypeOf([][Symbol.iterator]());

verifyProperty(ArrayIteratorProto.next, "name", {
  value: "next",
  writable: false,
  enumerable: false,
  configurable: true
});
