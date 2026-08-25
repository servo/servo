// Copyright (C) 2017 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-constructor
description: >
  Property descriptor of ArrayBuffer
info: |
  17 ECMAScript Standard Built-in Objects:

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
---*/

assert.sameValue(typeof ArrayBuffer, "function", "`typeof ArrayBuffer` is `'function'`");

verifyProperty(this, "ArrayBuffer", {
  writable: true,
  enumerable: false,
  configurable: true,
});
