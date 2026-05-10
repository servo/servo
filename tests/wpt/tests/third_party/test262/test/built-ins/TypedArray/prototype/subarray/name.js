// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.subarray
description: >
  %TypedArray%.prototype.subarray.name is "subarray".
info: |
  %TypedArray%.prototype.subarray( [ begin [ , end ] ] )

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

verifyProperty(TypedArray.prototype.subarray, "name", {
  value: "subarray",
  writable: false,
  enumerable: false,
  configurable: true
});
