// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.tostring
description: >
  Object.prototype.toString property descriptor
info: |
  17 ECMAScript Standard Built-in Objects:

  ...
  Every other data property described in clauses 18 through 26
  and in Annex B.2 has the attributes { [[Writable]]: true,
  [[Enumerable]]: false, [[Configurable]]: true } unless otherwise specified.

includes: [propertyHelper.js]
---*/

verifyProperty(Object.prototype, "toString", {
  writable: true,
  enumerable: false,
  configurable: true,
});
