// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal.plaintime.from step 2:
      2. Let _overflow_ be ? ToTemporalOverflow(_options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainTime(12),
  "12:00",
];
validValues.forEach((value) => {
  const explicit = Temporal.PlainTime.from(value, { overflow: undefined });
  TemporalHelpers.assertPlainTime(explicit, 12, 0, 0, 0, 0, 0, "overflow is ignored");
  const implicit = Temporal.PlainTime.from(value, {});
  TemporalHelpers.assertPlainTime(implicit, 12, 0, 0, 0, 0, 0, "overflow is ignored");
  const lambda = Temporal.PlainTime.from(value, () => {});
  TemporalHelpers.assertPlainTime(lambda, 12, 0, 0, 0, 0, 0, "overflow is ignored");
});

const propertyBag = { hour: 26 };
const explicit = Temporal.PlainTime.from(propertyBag, { overflow: undefined });
TemporalHelpers.assertPlainTime(explicit, 23, 0, 0, 0, 0, 0, "default overflow is constrain");
const implicit = Temporal.PlainTime.from(propertyBag, {});
TemporalHelpers.assertPlainTime(implicit, 23, 0, 0, 0, 0, 0, "default overflow is constrain");
const lambda = Temporal.PlainTime.from(propertyBag, () => {});
TemporalHelpers.assertPlainTime(lambda, 23, 0, 0, 0, 0, 0, "default overflow is constrain");
