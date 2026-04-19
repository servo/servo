// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Checks the propagation of exceptions from the options for the Segmenter constructor.
features: [Intl.Segmenter]
---*/

function CustomError() {}

const options = [
  "localeMatcher",
  "granularity",
];

for (const option of options) {
  assert.throws(CustomError, () => {
    new Intl.Segmenter("en", {
      get [option]() {
        throw new CustomError();
      }
    });
  }, `Exception from ${option} getter should be propagated`);
}
