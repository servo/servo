// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Type and property descriptor of Array.fromAsync
info: |
  Every other data property described in clauses 19 through 28 and in Annex B.2
  has the attributes { [[Writable]]: *true*, [[Enumerable]]: *false*,
  [[Configurable]]: *true* } unless otherwise specified.
includes: [propertyHelper.js]
features: [Array.fromAsync]
---*/

assert.sameValue(typeof Array.fromAsync, "function", "Array.fromAsync is callable");

verifyProperty(Array, 'fromAsync', {
  writable: true,
  enumerable: false,
  configurable: true,
});
