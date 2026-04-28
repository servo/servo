// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale
description: Checks the propagation of exceptions from the options for the Locale constructor.
features: [Intl.Locale]
---*/

function CustomError() {}

const options = [
  "language",
  "script",
  "region",
  "variants",
  "calendar",
  "collation",
  "hourCycle",
  "caseFirst",
  "numeric",
  "numberingSystem",
];

for (const option of options) {
  assert.throws(CustomError, () => {
    new Intl.Locale("en", {
      get [option]() {
        throw new CustomError();
      }
    });
  },
  `new Intl.Locale("en", {get ${option}() {throw new CustomError();}}) throws CustomError`);
}
