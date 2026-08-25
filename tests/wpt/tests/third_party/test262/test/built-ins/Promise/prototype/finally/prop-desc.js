// Copyright (C) 2017 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
author: Jordan Harband
description: Promise.prototype.finally property descriptor
esid: sec-promise.prototype.finally
info: |
    Every other data property described in clauses 18 through 26 and in Annex
    B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [Promise.prototype.finally]
---*/

assert.sameValue(typeof Promise.prototype.finally, 'function');

verifyProperty(Promise.prototype, 'finally', {
  writable: true,
  enumerable: false,
  configurable: true,
});
