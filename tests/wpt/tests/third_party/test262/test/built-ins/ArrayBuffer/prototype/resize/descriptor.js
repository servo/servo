// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.resize
description: >
  ArrayBuffer.prototype.resize has default data property attributes.
info: |
  ArrayBuffer.prototype.resize ( newLength )

  17 ECMAScript Standard Built-in Objects:
    Every other data property described in clauses 18 through 26 and in
    Annex B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [resizable-arraybuffer]
---*/

verifyProperty(ArrayBuffer.prototype, 'resize', {
  enumerable: false,
  writable: true,
  configurable: true
});
