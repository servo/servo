// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializepluralrules
description: Checks the propagation of exceptions from the options for the NumberFormat constructor.
---*/

function CustomError() {}

const options = [
  "localeMatcher",
  "type",
  "notation",
  "minimumIntegerDigits",
  "minimumFractionDigits",
  "maximumFractionDigits",
  "minimumSignificantDigits",
  "maximumSignificantDigits",
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
