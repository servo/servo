// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint16
description: >
  DataView.prototype.setInt16.name is "setInt16".
info: |
  DataView.prototype.setInt16 ( byteOffset, value [ , littleEndian ] )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(DataView.prototype.setInt16, "name", {
  value: "setInt16",
  writable: false,
  enumerable: false,
  configurable: true
});
