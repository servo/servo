// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-expanded-years
description: Negative zero, as an extended year, is rejected
info: |
  The year 0 is considered positive and must be prefixed with a + sign. The
  representation of the year 0 as -000000 is invalid.
---*/

const invalidStrings = [
  "-000000-03-31T00:45Z",
  "-000000-03-31T01:45",
  "-000000-03-31T01:45:00+01:00"
];

for (const str of invalidStrings) {
  assert.sameValue(Date.parse(str), NaN, "reject minus zero as extended year");
}
