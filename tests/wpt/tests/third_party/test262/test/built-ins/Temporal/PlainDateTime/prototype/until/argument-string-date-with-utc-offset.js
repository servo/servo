// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: UTC offset not valid with format that does not include a time
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const instance = new Temporal.PlainDateTime(1976, 11, 18, 15, 23);

const validStrings = [
  "1976-11-18T15:23+00",
  "1976-11-18T15:23+00:00",
  "1976-11-18T15:23+00:00:00,0",
  "1976-11-18T15:23+00:00:00.000000000",
  "1976-11-18T15:23+0000",
  "1976-11-18T15:23+000000,0",
  "1976-11-18T15:23+000000.000000000",
  "1976-11-18T15:23+00:00[UTC]",
  "1976-11-18T15:23+00:00[!UTC]",
  "1976-11-18T15:23+01[Europe/Vienna]",
  "1976-11-18T15:23-02:30[America/St_Johns]",
  "1976-11-18T15:23-02:30:00,0[America/St_Johns]",
  "1976-11-18T15:23-02:30:00.000000000[America/St_Johns]",
  "1976-11-18T15:23-0230[America/St_Johns]",
  "1976-11-18T15:23-023000,0[America/St_Johns]",
  "1976-11-18T15:23-023000.000000000[America/St_Johns]",
];

for (const arg of validStrings) {
  const result = instance.until(arg);

  TemporalHelpers.assertDuration(
    result,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `"${arg}" is a valid UTC offset with time for PlainDateTime`
  );
}

const invalidStrings = [
  "2022-09-15Z",
  "2022-09-15Z[UTC]",
  "2022-09-15Z[Europe/Vienna]",
  "2022-09-15+00:00",
  "2022-09-15+00:00[UTC]",
  "2022-09-15-02:30",
  "2022-09-15-02:30[America/St_Johns]",
];

for (const arg of invalidStrings) {
  assert.throws(
    RangeError,
    () => instance.until(arg),
    `"${arg}" UTC offset without time is not valid for PlainDateTime`
  );
}
