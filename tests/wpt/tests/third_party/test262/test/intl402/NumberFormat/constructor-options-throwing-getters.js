// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat
description: Checks the propagation of exceptions from the options for the NumberFormat constructor.
features: [Intl.NumberFormat-v3]
---*/

function CustomError() {}

const options = [
  "localeMatcher",
  "numberingSystem",
  "style",
  "currency",
  "currencyDisplay",
  "currencySign",
  "unit",
  "unitDisplay",
  "notation",
  "minimumIntegerDigits",
  "minimumFractionDigits",
  "maximumFractionDigits",
  "minimumSignificantDigits",
  "maximumSignificantDigits",
  "roundingIncrement",
  "roundingMode",
  "roundingPriority",
  "trailingZeroDisplay",
  "useGrouping",
  "compactDisplay",
  "signDisplay",
];

for (const option of options) {
  assert.throws(CustomError, () => {
    new Intl.NumberFormat("en", {
      get [option]() {
        throw new CustomError();
      }
    });
  }, `Exception from ${option} getter should be propagated`);
}
