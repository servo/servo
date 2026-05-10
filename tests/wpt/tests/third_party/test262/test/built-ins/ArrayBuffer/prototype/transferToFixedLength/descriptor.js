// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfertofixedlength
description: >
  ArrayBuffer.prototype.transferToFixedLength has default data property
  attributes.
info: |
  ArrayBuffer.prototype.transferToFixedLength ( [ newLength ] )

  17 ECMAScript Standard Built-in Objects:
    Every other data property described in clauses 18 through 26 and in
    Annex B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [arraybuffer-transfer]
---*/

verifyProperty(ArrayBuffer.prototype, 'transferToFixedLength', {
  enumerable: false,
  writable: true,
  configurable: true
});
