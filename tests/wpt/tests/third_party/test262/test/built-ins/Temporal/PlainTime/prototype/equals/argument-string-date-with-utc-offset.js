// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.equals
description: UTC offset not valid with format that does not include a time
features: [Temporal]
---*/

const instance = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);

const validStrings = [
  "12:34:56.987654321+00",
  "12:34:56.987654321+00:00",
  "12:34:56.987654321+00:00:00,0",
  "12:34:56.987654321+00:00:00.000000000",
  "12:34:56.987654321+0000",
  "12:34:56.987654321+000000,0",
  "12:34:56.987654321+000000.000000000",
  "12:34:56.987654321+00:00[UTC]",
  "12:34:56.987654321+00:00[!UTC]",
  "12:34:56.987654321+01[Europe/Vienna]",
  "12:34:56.987654321-02:30[America/St_Johns]",
  "12:34:56.987654321-02:30:00,0[America/St_Johns]",
  "12:34:56.987654321-02:30:00.000000000[America/St_Johns]",
  "12:34:56.987654321-0230[America/St_Johns]",
  "12:34:56.987654321-023000,0[America/St_Johns]",
  "12:34:56.987654321-023000.000000000[America/St_Johns]",
  "1976-11-18T12:34:56.987654321+00:00",
  "1976-11-18T12:34:56.987654321+00:00[UTC]",
  "1976-11-18T12:34:56.987654321+00:00[!UTC]",
  "1976-11-18T12:34:56.987654321-02:30[America/St_Johns]",
];

for (const arg of validStrings) {
  const result = instance.equals(arg);

  assert.sameValue(
    result,
    true,
    `"${arg}" is a valid UTC offset with time for PlainTime`
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
    () => instance.equals(arg),
    `"${arg}" UTC offset without time is not valid for PlainTime`
  );
}
