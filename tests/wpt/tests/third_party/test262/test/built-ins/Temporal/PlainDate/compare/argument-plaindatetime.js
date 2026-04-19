// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: Fast path for converting Temporal.PlainDateTime to Temporal.PlainDate by reading internal slots
info: |
    sec-temporal.plaindate.compare steps 1–2:
      1. Set _one_ to ? ToTemporalDate(_one_).
      2. Set _two_ to ? ToTemporalDate(_two_).
    sec-temporal-totemporaldate step 2.b:
      b. If _item_ has an [[InitializedTemporalDateTime]] internal slot, then
        i. Return ! CreateTemporalDate(_item_.[[ISOYear]], _item_.[[ISOMonth]], _item_.[[ISODay]], _item_.[[Calendar]]).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const sameDate = new Temporal.PlainDate(2000, 5, 2);
const earlierDate = new Temporal.PlainDate(1920, 7, 3);
const laterDate = new Temporal.PlainDate(2005, 1, 12);

TemporalHelpers.checkPlainDateTimeConversionFastPath((datetime) => {
  const result = Temporal.PlainDate.compare(datetime, sameDate);
  assert.sameValue(result, 0, "First argument, same date: comparison result");
}, "First argument, same date");

TemporalHelpers.checkPlainDateTimeConversionFastPath((datetime) => {
  const result = Temporal.PlainDate.compare(datetime, earlierDate);
  assert.sameValue(result, 1, "First argument, earlier date: comparison result");
}, "First argument, earlier date");

TemporalHelpers.checkPlainDateTimeConversionFastPath((datetime) => {
  const result = Temporal.PlainDate.compare(datetime, laterDate);
  assert.sameValue(result, -1, "First argument, later date: comparison result");
}, "First argument, later date");

TemporalHelpers.checkPlainDateTimeConversionFastPath((datetime) => {
  const result = Temporal.PlainDate.compare(sameDate, datetime);
  assert.sameValue(result, 0, "Second argument, same date: comparison result");
}, "Second argument, same date");

TemporalHelpers.checkPlainDateTimeConversionFastPath((datetime) => {
  const result = Temporal.PlainDate.compare(earlierDate, datetime);
  assert.sameValue(result, -1, "Second argument, earlier date: comparison result");
}, "Second argument, earlier date");

TemporalHelpers.checkPlainDateTimeConversionFastPath((datetime) => {
  const result = Temporal.PlainDate.compare(laterDate, datetime);
  assert.sameValue(result, 1, "Second argument, later date: comparison result");
}, "Second argument, later date");
