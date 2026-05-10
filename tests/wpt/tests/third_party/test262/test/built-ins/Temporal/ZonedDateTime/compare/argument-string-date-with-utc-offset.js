// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: UTC offset not valid with format that does not include a time
features: [Temporal]
---*/

const validStrings = [
  "1970-01-01T00Z[UTC]",
  "1970-01-01T00Z[!UTC]",
  "1970-01-01T00+00[UTC]",
  "1970-01-01T00+00:00[UTC]",
  "1970-01-01T00+00:00:00,0[UTC]",
  "1970-01-01T00+00:00:00.000000000[UTC]",
  "1970-01-01T00+0000[UTC]",
  "1970-01-01T00+000000,0[UTC]",
  "1970-01-01T00+000000.000000000[UTC]",
  "1970-01-01T00+00:00[!UTC]",
  "1970-01-01T00-00[UTC]",
  "1970-01-01T00-00:00[UTC]",
  "1970-01-01T00-00:00:00,0[UTC]",
  "1970-01-01T00-00:00:00.000000000[UTC]",
  "1970-01-01T00-0000[UTC]",
  "1970-01-01T00-000000,0[UTC]",
  "1970-01-01T00-000000.000000000[UTC]",
];

for (const arg of validStrings) {
  const result = Temporal.ZonedDateTime.compare(arg, arg);

  assert.sameValue(
    result,
    0,
    `"${arg}" is a valid UTC offset with time for ZonedDateTime`
  );
}

const invalidStrings = [
  "2022-09-15Z[UTC]",
  "2022-09-15Z[Europe/Vienna]",
  "2022-09-15+00:00[UTC]",
  "2022-09-15-02:30[America/St_Johns]",
];
const datetime = new Temporal.ZonedDateTime(0n, "UTC");

for (const arg of invalidStrings) {
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.compare(arg, datetime),
    `"${arg}" UTC offset without time is not valid for ZonedDateTime (first argument)`
  );
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.compare(datetime, arg),
    `"${arg}" UTC offset without time is not valid for ZonedDateTime (second argument)`
  );
}
