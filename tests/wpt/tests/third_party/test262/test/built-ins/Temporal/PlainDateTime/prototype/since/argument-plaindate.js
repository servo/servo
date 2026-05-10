// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Fast path for converting Temporal.PlainDate to Temporal.PlainDateTime by reading internal slots
info: |
    sec-temporal.plaindatetime.prototype.since step 3:
      3. Set _other_ to ? ToTemporalDateTime(_other_).
    sec-temporal-totemporaldatetime step 2.b:
      b. If _item_ has an [[InitializedTemporalDate]] internal slot, then
        i. Return ? CreateTemporalDateTime(_item_.[[ISOYear]], _item_.[[ISOMonth]], _item_.[[ISODay]], 0, 0, 0, 0, 0, 0, _item_.[[Calendar]]).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkToTemporalPlainDateTimeFastPath((date, calendar) => {
  const datetime = new Temporal.PlainDateTime(2000, 5, 2, 0, 0, 0, 987, 654, 321, calendar);
  const result = datetime.since(date);
  assert.sameValue(result.total({ unit: "nanoseconds" }), 987654321, "PlainDate is converted to midnight");
});
