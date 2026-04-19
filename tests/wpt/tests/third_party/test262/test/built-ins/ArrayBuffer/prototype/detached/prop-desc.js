// Copyright (C) 2023 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-arraybuffer.prototype.detached
description: >
  "detached" property of ArrayBuffer.prototype
info: |
  ArrayBuffer.prototype.detached is an accessor property whose set
  accessor function is undefined.

  Section 17: Every accessor property described in clauses 18 through 26 and in
  Annex B.2 has the attributes {[[Enumerable]]: false, [[Configurable]]: true }
includes: [propertyHelper.js]
features: [ArrayBuffer, arraybuffer-transfer]
---*/

var desc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, 'detached');

assert.sameValue(desc.set, undefined);
assert.sameValue(typeof desc.get, 'function');

verifyProperty(ArrayBuffer.prototype, 'detached', {
  enumerable: false,
  configurable: true
});
