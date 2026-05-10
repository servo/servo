// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-additional-properties-of-the-object.prototype-object
description: Property descriptor for Object.prototype.__lookupSetter__
info: |
    Every other data property described in clauses 18 through 26 and in Annex
    B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [__setter__]
---*/

verifyProperty(Object.prototype, "__lookupSetter__", {
  enumerable: false,
  writable: true,
  configurable: true,
});
