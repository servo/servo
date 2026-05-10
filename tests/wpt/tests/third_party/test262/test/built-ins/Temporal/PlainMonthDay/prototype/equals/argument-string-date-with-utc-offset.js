// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.equals
description: UTC offset not valid with format that does not include a time
features: [Temporal]
---*/

const instance = new Temporal.PlainMonthDay(5, 2);

const validStrings = [
  "05-02[Asia/Katmandu]",
  "05-02[!Asia/Katmandu]",
  "05-02[u-ca=iso8601]",
  "05-02[Asia/Tokyo][u-ca=iso8601]",
  "--05-02[Asia/Katmandu]",
  "--05-02[!Asia/Katmandu]",
  "--05-02[u-ca=iso8601]",
  "--05-02[Asia/Tokyo][u-ca=iso8601]",
  "2000-05-02T00+00",
  "2000-05-02T00+00:00",
  "2000-05-02T00+00:00:00,0",
  "2000-05-02T00+00:00:00.000000000",
  "2000-05-02T00+0000",
  "2000-05-02T00+000000,0",
  "2000-05-02T00+000000.000000000",
  "2000-05-02T00+00:00[UTC]",
  "2000-05-02T00+01[Europe/Vienna]",
  "2000-05-02T00-02:30[America/St_Johns]",
  "2000-05-02T00-02:30:00,0[America/St_Johns]",
  "2000-05-02T00-02:30:00.000000000[America/St_Johns]",
  "2000-05-02T00-0230[America/St_Johns]",
  "2000-05-02T00-023000,0[America/St_Johns]",
  "2000-05-02T00-023000.000000000[America/St_Johns]",
];

for (const arg of validStrings) {
  const result = instance.equals(arg);

  assert.sameValue(
    result,
    true,
    `"${arg}" is a valid UTC offset with time for PlainMonthDay`
  );
}

const invalidStrings = [
  "09-15Z",
  "09-15Z[UTC]",
  "09-15+01:00",
  "09-15+01:00[Europe/Vienna]",
  "--09-15Z",
  "--09-15Z[UTC]",
  "--09-15+01:00",
  "--09-15+01:00[Europe/Vienna]",
  "2022-09-15Z",
  "2022-09-15Z[UTC]",
  "2022-09-15Z[Europe/Vienna]",
  "2022-09-15+00:00",
  "2022-09-15+00:00[UTC]",
  "2022-09-15-02:30",
  "2022-09-15-02:30[America/St_Johns]",
  "09-15[u-ca=chinese]",
];

for (const arg of invalidStrings) {
  assert.throws(
    RangeError,
    () => instance.equals(arg),
    `"${arg}" UTC offset without time is not valid for PlainMonthDay`
  );
}
