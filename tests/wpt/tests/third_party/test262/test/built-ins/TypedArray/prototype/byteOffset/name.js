// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-%typedarray%.prototype.byteoffset
description: >
  get %TypedArray%.prototype.byteOffset.name is "get byteOffset".
info: |
  get %TypedArray%.prototype.byteOffset

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js, testTypedArray.js]
features: [TypedArray]
---*/

var desc = Object.getOwnPropertyDescriptor(TypedArray.prototype, "byteOffset");

verifyProperty(desc.get, "name", {
  value: "get byteOffset",
  writable: false,
  enumerable: false,
  configurable: true
});
