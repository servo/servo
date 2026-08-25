// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.findlast
description: >
  %TypedArray%.prototype.findLast.name is "findLast".
info: |
  %TypedArray%.prototype.findLast (predicate [ , thisArg ] )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js, testTypedArray.js]
features: [TypedArray, array-find-from-last]
---*/

assert.sameValue(TypedArray.prototype.findLast.name, "findLast");

verifyProperty(TypedArray.prototype.findLast, "name", {
  enumerable: false,
  writable: false,
  configurable: true
});
