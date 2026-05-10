// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-interprettemporaldatetimefields steps 2–3:
      2. Let _temporalDate_ be ? DateFromFields(_calendar_, _fields_, _options_).
      3. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal-totemporalzoneddatetime steps 2–3:
      2. If Type(_item_) is Object, then
        ...
        j. Let _result_ be ? InterpretTemporalDateTimeFields(_calendar_, _fields_, _options_).
      3. Else,
        a. Perform ? ToTemporalOverflow(_options_).
    sec-temporal.zoneddatetime.from steps 2–3:
      2. If Type(_item_) is Object and _item_ has an [[InitializedTemporalZonedDateTime]] internal slot, then
        a. Perform ? ToTemporalOverflow(_options_).
        ...
        d. Return ...
      3. Return ? ToTemporalZonedDateTime(_item_, _options_).
features: [Temporal]
---*/

const validValues = [
  new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC"),
  "2001-09-09T01:46:40.987654321+00:00[UTC]",
];
validValues.forEach((value) => {
  const explicit = Temporal.ZonedDateTime.from(value, { overflow: undefined });
  assert.sameValue(explicit.epochNanoseconds, 1_000_000_000_987_654_321n, "overflow is ignored");
  const implicit = Temporal.ZonedDateTime.from(value, {});
  assert.sameValue(implicit.epochNanoseconds, 1_000_000_000_987_654_321n, "overflow is ignored");
  const lambda = Temporal.ZonedDateTime.from(value, () => {});
  assert.sameValue(lambda.epochNanoseconds, 1_000_000_000_987_654_321n, "overflow is ignored");
});

const propertyBag = { year: 2000, month: 15, day: 34, hour: 12, timeZone: "UTC" };
const explicit = Temporal.ZonedDateTime.from(propertyBag, { overflow: undefined });
assert.sameValue(explicit.epochNanoseconds, 978_264_000_000_000_000n, "default overflow is constrain");

// See options-object.js for {} and () => {}
