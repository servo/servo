// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat.prototype.resolvedoptions
description: >
  "resolvedOptions" property of Intl.DateTimeFormat.prototype.
info: |
  Intl.DateTimeFormat.prototype.resolvedOptions ()

  7 Requirements for Standard Built-in ECMAScript Objects

    Unless specified otherwise in this document, the objects, functions, and constructors
    described in this standard are subject to the generic requirements and restrictions
    specified for standard built-in ECMAScript objects in the ECMAScript 2018 Language
    Specification, 9th edition, clause 17, or successor.

  17 ECMAScript Standard Built-in Objects:

    Every other data property described in clauses 18 through 26 and in Annex B.2 has the
    attributes { [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }
    unless otherwise specified.

includes: [propertyHelper.js]
---*/

verifyProperty(Intl.DateTimeFormat.prototype, "resolvedOptions", {
  writable: true,
  enumerable: false,
  configurable: true,
});
