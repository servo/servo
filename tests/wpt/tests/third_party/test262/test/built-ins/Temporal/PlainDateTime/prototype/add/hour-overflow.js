// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Testing overflow hours (adding hours that push one to the next/previous day)
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const earlier = new Temporal.PlainDateTime(2020, 5, 31, 23, 12, 38, 271, 986, 102);

TemporalHelpers.assertPlainDateTime(
  earlier.add({ hours: 2 }),
  2020, 6, "M06", 1, 1, 12, 38, 271, 986, 102,
  "hours overflow (push to next day)"
);

const later = new Temporal.PlainDateTime(2019, 10, 29, 10, 46, 38, 271, 986, 102);

TemporalHelpers.assertPlainDateTime(
  later.add({ hours: -12 }),
  2019, 10, "M10", 28, 22, 46, 38, 271, 986, 102,
  "hours overflow (push to previous day)"
);
