// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Fallback value for offset option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
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

const propertyBag = { timeZone: "-04:00", offset: "+01:00", year: 2020, month: 2, day: 16, hour: 23, minute: 45 };

assert.throws(RangeError, () => Temporal.ZonedDateTime.from(propertyBag, { offset: undefined }), "default offset is reject");
// See options-object.js for {} and () => {}
