// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 22.2.2.4
description: >
  get %TypedArray% [ @@species ].name is "get [Symbol.species]".
info: |
  get %TypedArray% [ @@species ]

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js, testTypedArray.js]
features: [Symbol.species]
---*/

var desc = Object.getOwnPropertyDescriptor(TypedArray, Symbol.species);

verifyProperty(desc.get, "name", {
  value: "get [Symbol.species]",
  writable: false,
  enumerable: false,
  configurable: true
});
