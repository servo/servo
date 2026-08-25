// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: By default, overflow = constrain
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const date = {year: 2019, month: 1, day: 32};
const data = [2019, 1, "M01", 31, 0, 0, 0, 0, 0, 0];

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from(date),
  ...data,
  "by default, overflow is constrain (overflow options argument absent)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from(date, {}),
  ...data,
  "by default, overflow is constrain (options argument is empty plain object)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from(date, () => {}),
  ...data,
  "by default, overflow is constrain (options argument is empty function)"
);


TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from(date, {overflow: "constrain"}),
  ...data,
  "by default, overflow is constrain (overflow options argument present)"
);
