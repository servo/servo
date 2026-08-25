// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.compare
description: Plain object arguments may throw if they do not contain sufficient information
features: [Temporal]
---*/

const dt1 = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);
const dt2 = new Temporal.PlainDateTime(2019, 10, 29, 10, 46, 38, 271, 986, 102);

assert.throws(
  TypeError,
  () => Temporal.PlainDateTime.compare({ year: 1976 }, dt2),
  "object must contain at least the required properties (first arg)"
);

assert.throws(
  TypeError,
  () => Temporal.PlainDateTime.compare(dt1, { year: 2019 }),
  "object must contain at least the required properties (second arg)"
);
