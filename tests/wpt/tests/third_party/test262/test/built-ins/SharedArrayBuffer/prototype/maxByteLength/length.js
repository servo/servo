// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-sharedarraybuffer.prototype.maxbytelength
description: >
  get SharedArrayBuffer.prototype.maxByteLength.length is 0.
info: |
  get SharedArrayBuffer.prototype.maxByteLength

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
features: [SharedArrayBuffer, resizable-arraybuffer]
---*/

var desc = Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype, 'maxByteLength');

verifyProperty(desc.get, 'length', {
  value: 0,
  enumerable: false,
  writable: false,
  configurable: true
});
