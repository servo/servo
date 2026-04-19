// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: Atomics.waitAsync property descriptor
info: |
  ES Section 17

  Every other data property described in clauses 18 through 26 and in Annex
  B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [Atomics.waitAsync, Atomics]
---*/

verifyProperty(Atomics, 'waitAsync', {
  configurable: true,
  writable: true,
  enumerable: false,
});
