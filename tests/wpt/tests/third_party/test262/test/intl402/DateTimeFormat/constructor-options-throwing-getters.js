// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: Checks the propagation of exceptions from the options for the DateTimeFormat constructor.
---*/

function CustomError() {}

const options = [
  "weekday", "year", "month", "day",
  "hour", "minute", "second",
  "localeMatcher",
  "hour12",
  "hourCycle",
  "timeZone",
  "era",
  "timeZoneName",
  "formatMatcher",
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
