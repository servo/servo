// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Fuzzy matching behaviour with UTC offsets in ISO 8601 strings with named time zones and offset option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = Temporal.ZonedDateTime.from({ year: 1970, month: 1, day: 1, hour: 12, timeZone: "Africa/Monrovia" });
assert.sameValue(instance.offset, "-00:44:30", "original offset");
const properties = { day: 2, offset: "-00:45" };

["ignore", "prefer"].forEach((offset) => {
  const result = instance.with(properties, { offset });
  assert.sameValue(result.epochNanoseconds, 132270_000_000_000n, `ignores new offset (offset=${offset})`);
  assert.sameValue(result.offset, instance.offset, "offset property is unchanged");
  TemporalHelpers.assertPlainDateTime(
    result.toPlainDateTime(),
    1970,
    1,
    "M01",
    2,
    12,
    0,
    0,
    0,
    0,
    0,
    "wall time is not shifted"
  );
});

const result = instance.with(properties, { offset: "use" });
assert.sameValue(result.epochNanoseconds, 132300_000_000_000n, "accepts HH:MM rounded offset (offset=use)");
assert.sameValue(result.offset, instance.offset, "offset property is unchanged");
TemporalHelpers.assertPlainDateTime(
  result.toPlainDateTime(),
  1970,
  1,
  "M01",
  2,
  12,
  0,
  30,
  0,
  0,
  0,
  "wall time is shifted by the difference between exact and rounded offset"
);

assert.throws(RangeError, () => instance.with(properties, { offset: "reject" }), "no fuzzy matching is done in with()");
