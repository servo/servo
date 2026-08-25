// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Fast path for converting Temporal.PlainDateTime to Temporal.PlainDate by reading internal slots
info: |
    sec-temporal.plaindate.from step 3:
      3. Return ? ToTemporalDate(_item_, _options_).
    sec-temporal-totemporaldate step 2.b:
      b. If _item_ has an [[InitializedTemporalDateTime]] internal slot, then
        i. Return ! CreateTemporalDate(_item_.[[ISOYear]], _item_.[[ISOMonth]], _item_.[[ISODay]], _item_.[[Calendar]]).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkPlainDateTimeConversionFastPath((datetime) => {
  const result = Temporal.PlainDate.from(datetime);
  TemporalHelpers.assertPlainDate(result, 2000, 5, "M05", 2);
});
