// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: Checks the propagation of exceptions from the options for the DateTimeFormat constructor.
features: [Intl.DateTimeFormat-datetimestyle]
---*/

// To be merged into constructor-options-throwing-getters.js when the feature is removed.

function CustomError() {}

const options = [
  // CreateDateTimeFormat step 39 
  "dateStyle",
  // CreateDateTimeFormat step 41 
  "timeStyle",
];

for (const option of options) {
  assert.throws(CustomError, () => {
    new Intl.DateTimeFormat("en", {
      get [option]() {
        throw new CustomError();
      }
    });
  }, `Exception from ${option} getter should be propagated`);
}
