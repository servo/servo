// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Checks the propagation of exceptions from the options for the ListFormat constructor.
features: [Intl.ListFormat]
---*/

function CustomError() {}

const options = [
  "type",
  "style",
];

for (const option of options) {
  assert.throws(CustomError, () => {
    new Intl.ListFormat("en", {
      get [option]() {
        throw new CustomError();
      }
    });
  }, `Exception from ${option} getter should be propagated`);
}
