// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfer
description: >
  ArrayBuffer.prototype.transfer.name is "transfer".
info: |
  ArrayBuffer.prototype.transfer ( [ newLength ] )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
features: [arraybuffer-transfer]
includes: [propertyHelper.js]
---*/

verifyProperty(ArrayBuffer.prototype.transfer, 'name', {
  value: 'transfer',
  enumerable: false,
  writable: false,
  configurable: true
});
