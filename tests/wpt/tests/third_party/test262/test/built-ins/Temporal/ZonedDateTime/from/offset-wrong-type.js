// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Type conversions for offset option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloffset step 1:
      1. Return ? GetOption(_normalizedOptions_, *"offset"*, « String », « *"prefer"*, *"use"*, *"ignore"*, *"reject"* », _fallback_).
    sec-temporal-totemporalzoneddatetime step 6:
      6. Let _offset_ be ? ToTemporalOffset(_options_, *"reject"*).
    sec-temporal.zoneddatetime.from step 2:
      2. If Type(_item_) is Object and _item_ has an [[InitializedTemporalZonedDateTime]] internal slot, then
        ...
        c. Perform ? ToTemporalOffset(_options_, *"reject"*).
        d. Return ...
      3. Return ? ToTemporalZonedDateTime(_item_, _options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");
TemporalHelpers.checkStringOptionWrongType("offset", "reject",
  (offset) => Temporal.ZonedDateTime.from(datetime, { offset }),
  (result, descr) => assert.sameValue(result.epochNanoseconds, 1_000_000_000_987_654_321n, descr),
);

const propertyBag = { timeZone: "UTC", offset: "+00:00", year: 2001, month: 9, day: 9, hour: 1, minute: 46, second: 40, millisecond: 987, microsecond: 654, nanosecond: 321 };
TemporalHelpers.checkStringOptionWrongType("offset", "reject",
  (offset) => Temporal.ZonedDateTime.from(propertyBag, { offset }),
  (result, descr) => assert.sameValue(result.epochNanoseconds, 1_000_000_000_987_654_321n, descr),
);
