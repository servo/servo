// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.constructor
description: >
    Checks the "constructor" property of the Locale prototype object.
info: |
    Intl.Locale.prototype.constructor

    The initial value of Intl.Locale.prototype.constructor is %Locale%.

    Unless specified otherwise in this document, the objects, functions, and constructors described in this standard are subject to the generic requirements and restrictions specified for standard built-in ECMAScript objects in the ECMAScript 2019 Language Specification, 10th edition, clause 17, or successor.

    Every other data property described in clauses 18 through 26 and in Annex B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [Intl.Locale]
---*/

verifyProperty(Intl.Locale.prototype, 'constructor', {
  value: Intl.Locale,
  writable: true,
  enumerable: false,
  configurable: true,
});
