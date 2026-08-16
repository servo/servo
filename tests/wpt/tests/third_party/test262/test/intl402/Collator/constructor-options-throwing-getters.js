// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.collator
description: Checks the propagation of exceptions from the options for the Collator constructor.
locale: [en]
---*/

function CustomError() {}

const options = [
  "usage",
  "localeMatcher",
  "collation",
  "numeric",
  "caseFirst",
  "sensitivity",
  "ignorePunctuation",
];

for (const option of options) {
  assert.throws(CustomError, () => {
    new Intl.Collator("en", {
      get [option]() {
        throw new CustomError();
      }
    });
  }, `Exception from ${option} getter should be propagated`);
}
