// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Property descriptor of Iterator.prototype.chunks
info: |
  Iterator.prototype.chunks

  17 ECMAScript Standard Built-in Objects

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
features: [iterator-chunking]
includes: [propertyHelper.js]
---*/

verifyProperty(Iterator.prototype, 'chunks', {
  writable: true,
  enumerable: false,
  configurable: true,
});
