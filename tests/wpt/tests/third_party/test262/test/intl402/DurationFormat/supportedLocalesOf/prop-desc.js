// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.supportedLocalesOf
description: >
    Checks the "supportedLocalesOf" property of the DurationFormat prototype object.
info: |
    Intl.DurationFormat.supportedLocalesOf ( locales [, options ])

    Unless specified otherwise in this document, the objects, functions, and constructors described in this standard are subject to the generic requirements and restrictions specified for standard built-in ECMAScript objects in the ECMAScript 2019 Language Specification, 10th edition, clause 17, or successor.

    Every other data property described in clauses 18 through 26 and in Annex B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [Intl.DurationFormat]
---*/

assert.sameValue(
  typeof Intl.DurationFormat.supportedLocalesOf,
  "function",
  "typeof Intl.DurationFormat.supportedLocalesOf is function"
);

verifyProperty(Intl.DurationFormat, "supportedLocalesOf", {
  writable: true,
  enumerable: false,
  configurable: true,
});

