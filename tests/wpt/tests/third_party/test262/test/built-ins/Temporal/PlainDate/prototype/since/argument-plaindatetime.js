// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.since
description: Fast path for converting Temporal.PlainDateTime to Temporal.PlainDate by reading internal slots
info: |
    sec-temporal.plaindate.prototype.since step 3:
      3. Set _other_ to ? ToTemporalDate(_other_).
    sec-temporal-totemporaldate step 2.b:
      b. If _item_ has an [[InitializedTemporalDateTime]] internal slot, then
        i. Return ! CreateTemporalDate(_item_.[[ISOYear]], _item_.[[ISOMonth]], _item_.[[ISODay]], _item_.[[Calendar]]).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkPlainDateTimeConversionFastPath((datetime) => {
  const date = new Temporal.PlainDate(2000, 5, 2);
  const result = date.since(datetime);
  assert.sameValue(result.total({ unit: "nanoseconds" }), 0, "time part dropped");
});
