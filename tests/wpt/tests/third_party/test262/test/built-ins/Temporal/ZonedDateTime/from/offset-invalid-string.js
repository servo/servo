// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: RangeError thrown when offset option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
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
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(datetime, { offset: "other string" }));

const propertyBag = { timeZone: "UTC", year: 2001, month: 9, day: 9, hour: 1, minute: 46, second: 40, millisecond: 987, microsecond: 654, nanosecond: 321 };
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(propertyBag, { offset: "other string" }));
