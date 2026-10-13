// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.pluralrules
description: >
  Checks the propagation of exceptions from the options for the PluralRules
  constructor.
locale: [en]
---*/

function CustomError() {}

const options = [
  "localeMatcher",
  "type",
  "notation",
  "compactDisplay",
  "minimumIntegerDigits",
  "minimumFractionDigits",
  "maximumFractionDigits",
  "minimumSignificantDigits",
  "maximumSignificantDigits",
  "roundingIncrement",
  "roundingMode",
  "roundingPriority",
  "trailingZeroDisplay",
];

for (const option of options) {
  assert.throws(CustomError, () => {
    new Intl.PluralRules("en", {
      get [option]() {
        throw new CustomError();
      }
    });
  }, `Exception from ${option} getter should be propagated`);
}
