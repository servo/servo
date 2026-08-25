// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.resize
description: >
  ArrayBuffer.prototype.resize.length is 1.
info: |
  ArrayBuffer.prototype.resize ( newLength )

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
features: [resizable-arraybuffer]
---*/

verifyProperty(ArrayBuffer.prototype.resize, 'length', {
  value: 1,
  enumerable: false,
  writable: false,
  configurable: true
});
