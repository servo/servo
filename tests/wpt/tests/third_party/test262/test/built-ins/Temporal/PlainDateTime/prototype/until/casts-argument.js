// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: String and object arguments get cast
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);

TemporalHelpers.assertDuration(
  datetime.until({ year: 2019, month: 10, day: 29, hour: 10 }),
  0, 0, 0, 15684, 18, 36, 29, 876, 543, 211,
  "plain object argument"
);

TemporalHelpers.assertDuration(
  datetime.until("2019-10-29T10:46:38.271986102"),
  0, 0, 0, 15684, 19, 23, 8, 148, 529, 313,
  "string argument gets cast"
);
