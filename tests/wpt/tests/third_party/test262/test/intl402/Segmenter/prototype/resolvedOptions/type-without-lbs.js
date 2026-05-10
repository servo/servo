// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.prototype.resolvedOptions
description: Checks the properties of the result of Intl.Segmenter.prototype.resolvedOptions().
info: |
    Intl.Segmenter.prototype.resolvedOptions ()

    3. Let options be ! ObjectCreate(%ObjectPrototype%).
    4. For each row of Table 1, except the header row, do
        c. If v is not undefined, then
            i. Perform ! CreateDataPropertyOrThrow(options, p, v).
includes: [propertyHelper.js]
features: [Intl.Segmenter]
---*/

const rtf = new Intl.Segmenter("en-us", { "lineBreakStyle": "loose", "granularity": "word" });
const options = rtf.resolvedOptions();
assert.sameValue(Object.getPrototypeOf(options), Object.prototype, "Prototype");

verifyProperty(options, "locale", {
  value: "en-US",
  writable: true,
  enumerable: true,
  configurable: true,
});

verifyProperty(options, "granularity", {
  value: "word",
  writable: true,
  enumerable: true,
  configurable: true,
});

verifyProperty(options, "lineBreakStyle", undefined);
