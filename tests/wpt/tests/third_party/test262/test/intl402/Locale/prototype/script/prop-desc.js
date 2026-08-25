// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks the "script" property of the Locale prototype object.
info: |
    Intl.Locale.prototype.script

    Unless specified otherwise in this document, the objects, functions, and constructors described in this standard are subject to the generic requirements and restrictions specified for standard built-in ECMAScript objects in the ECMAScript 2019 script Specification, 10th edition, clause 17, or successor.

    Every accessor property described in clauses 18 through 26 and in Annex B.2 has the attributes { [[Enumerable]]: false, [[Configurable]]: true } unless otherwise specified. If only a get accessor function is described, the set accessor function is the default value, undefined.
includes: [propertyHelper.js]
features: [Intl.Locale]
---*/

const propdesc = Object.getOwnPropertyDescriptor(Intl.Locale.prototype, "script");
assert.sameValue(propdesc.set, undefined);
assert.sameValue(typeof propdesc.get, "function");

verifyProperty(Intl.Locale.prototype, "script", {
  enumerable: false,
  configurable: true,
});
