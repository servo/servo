// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: RangeError thrown when disambiguation option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
    sec-temporal-totemporaldisambiguation step 1:
      1. Return ? GetOption(_normalizedOptions_, *"disambiguation"*, « String », « *"compatible"*, *"earlier"*, *"later"*, *"reject"* », *"compatible"*).
    sec-temporal-totemporalzoneddatetime step 5:
      5. Let _disambiguation_ be ? ToTemporalDisambiguation(_options_).
    sec-temporal.zoneddatetime.from step 2:
      2. If Type(_item_) is Object and _item_ has an [[InitializedTemporalZonedDateTime]] internal slot, then
        a. ...
        b. Perform ? ToTemporalDisambiguation(_options_).
        c. ...
        d. Return ...
      3. Return ? ToTemporalZonedDateTime(_item_, _options_).
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(datetime, { disambiguation: "other string" }));

const propertyBag = { timeZone: "UTC", year: 2001, month: 9, day: 9, hour: 1, minute: 46, second: 40, millisecond: 987, microsecond: 654, nanosecond: 321 };
[
    "",
    "EARLIER",
    "balance",
    "other string",
    3,
    null
].forEach(disambiguation => {
    assert.throws(RangeError, () => Temporal.ZonedDateTime.from(propertyBag, { disambiguation }))
});
