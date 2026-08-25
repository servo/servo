// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: Fast path for converting Temporal.PlainDate to Temporal.PlainDateTime by reading internal slots
info: |
    sec-temporal.plaindatetime.prototype.equals step 3:
      3. Set _other_ to ? ToTemporalDateTime(_other_).
    sec-temporal-totemporaldatetime step 2.b:
      b. If _item_ has an [[InitializedTemporalDate]] internal slot, then
        i. Return ? CreateTemporalDateTime(_item_.[[ISOYear]], _item_.[[ISOMonth]], _item_.[[ISODay]], 0, 0, 0, 0, 0, 0, _item_.[[Calendar]]).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkToTemporalPlainDateTimeFastPath((date, calendar) => {
  const datetime = new Temporal.PlainDateTime(2000, 5, 2, 0, 0, 0, 0, 0, 0, calendar);
  assert(datetime.equals(date));
});
