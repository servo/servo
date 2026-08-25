// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: >
  "format" property of Intl.NumberFormat.prototype.
info: |
  get Intl.NumberFormat.prototype.format

  7 Requirements for Standard Built-in ECMAScript Objects

    Unless specified otherwise in this document, the objects, functions, and constructors
    described in this standard are subject to the generic requirements and restrictions
    specified for standard built-in ECMAScript objects in the ECMAScript 2018 Language
    Specification, 9th edition, clause 17, or successor.

  17 ECMAScript Standard Built-in Objects:

    Every accessor property described in clauses 18 through 26 and in Annex B.2 has the
    attributes { [[Enumerable]]: false, [[Configurable]]: true } unless otherwise specified.
    If only a get accessor function is described, the set accessor function is the default
    value, undefined. If only a set accessor is described the get accessor is the default
    value, undefined.

includes: [propertyHelper.js]
---*/

var desc = Object.getOwnPropertyDescriptor(Intl.NumberFormat.prototype, "format");

assert.sameValue(desc.set, undefined);
assert.sameValue(typeof desc.get, "function");

verifyProperty(Intl.NumberFormat.prototype, "format", {
  enumerable: false,
  configurable: true,
});
