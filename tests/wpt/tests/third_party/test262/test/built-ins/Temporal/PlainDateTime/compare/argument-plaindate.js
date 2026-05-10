// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.compare
description: Fast path for converting Temporal.PlainDate to Temporal.PlainDateTime by reading internal slots
info: |
    sec-temporal.plaindatetime.compare steps 1–2:
      1. Set _one_ to ? ToTemporalDateTime(_one_).
      2. Set _two_ to ? ToTemporalDateTime(_two_).
    sec-temporal-totemporaldatetime step 2.b:
      b. If _item_ has an [[InitializedTemporalDate]] internal slot, then
        i. Return ? CreateTemporalDateTime(_item_.[[ISOYear]], _item_.[[ISOMonth]], _item_.[[ISODay]], 0, 0, 0, 0, 0, 0, _item_.[[Calendar]]).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);

TemporalHelpers.checkToTemporalPlainDateTimeFastPath((date) => {
  const result = Temporal.PlainDateTime.compare(date, datetime);
  assert.sameValue(result, -1, "PlainDate is converted to midnight");
});

TemporalHelpers.checkToTemporalPlainDateTimeFastPath((date) => {
  const result = Temporal.PlainDateTime.compare(datetime, date);
  assert.sameValue(result, 1, "PlainDate is converted to midnight");
});
