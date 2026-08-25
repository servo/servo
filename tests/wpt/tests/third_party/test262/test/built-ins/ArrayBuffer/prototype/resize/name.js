// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.resize
description: >
  ArrayBuffer.prototype.resize.name is "resize".
info: |
  ArrayBuffer.prototype.resize ( newLength )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
features: [resizable-arraybuffer]
includes: [propertyHelper.js]
---*/

verifyProperty(ArrayBuffer.prototype.resize, 'name', {
  value: 'resize',
  enumerable: false,
  writable: false,
  configurable: true
});
