// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const overflowFields = { year: 2000, month: 13, day: 2, timeZone: "UTC" };

const overflowExplicit = Temporal.ZonedDateTime.from(overflowFields, undefined);
assert.sameValue(overflowExplicit.month, 12, "default overflow is constrain");

const overflowPropertyImplicit = Temporal.ZonedDateTime.from(overflowFields, {});
assert.sameValue(overflowPropertyImplicit.month, 12, "default overflow is constrain");

const overflowImplicit = Temporal.ZonedDateTime.from(overflowFields);
assert.sameValue(overflowImplicit.month, 12, "default overflow is constrain");

const timeZone = "America/Vancouver";
const disambiguationEarlierFields = { timeZone, year: 2000, month: 10, day: 29, hour: 1, minute: 34, second: 56, millisecond: 987, microsecond: 654, nanosecond: 321 };
const disambiguationLaterFields = { timeZone, year: 2000, month: 4, day: 2, hour: 2, minute: 34, second: 56, millisecond: 987, microsecond: 654, nanosecond: 321 };

[
  [disambiguationEarlierFields, 972808496987654321n],
  [disambiguationLaterFields, 954671696987654321n],
].forEach(([fields, expected]) => {
  const explicit = Temporal.ZonedDateTime.from(fields, undefined);
  assert.sameValue(explicit.epochNanoseconds, expected, "default disambiguation is compatible");

  const propertyImplicit = Temporal.ZonedDateTime.from(fields, {});
  assert.sameValue(propertyImplicit.epochNanoseconds, expected, "default disambiguation is compatible");

  const implicit = Temporal.ZonedDateTime.from(fields);
  assert.sameValue(implicit.epochNanoseconds, expected, "default disambiguation is compatible");
});

const offsetFields = { year: 2000, month: 5, day: 2, offset: "+23:59", timeZone: "UTC" };
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(offsetFields, undefined), "default offset is reject");
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(offsetFields, {}), "default offset is reject");
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(offsetFields), "default offset is reject");
