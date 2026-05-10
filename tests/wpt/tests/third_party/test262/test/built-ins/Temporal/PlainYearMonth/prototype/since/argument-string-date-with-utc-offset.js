// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: UTC offset not valid with format that does not include a time
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const instance = new Temporal.PlainYearMonth(2019, 12);

const validStrings = [
  "2019-12[Africa/Abidjan]",
  "2019-12[!Africa/Abidjan]",
  "2019-12[u-ca=iso8601]",
  "2019-12[Africa/Abidjan][u-ca=iso8601]",
  "2019-12-15T00+00",
  "2019-12-15T00+00:00",
  "2019-12-15T00+00:00:00,0",
  "2019-12-15T00+00:00:00.000000000",
  "2019-12-15T00+0000",
  "2019-12-15T00+000000,0",
  "2019-12-15T00+000000.000000000",
  "2019-12-15T00+00:00[UTC]",
  "2019-12-15T00+00:00[!UTC]",
  "2019-12-15T00+01[Europe/Vienna]",
  "2019-12-15T00-02:30[America/St_Johns]",
  "2019-12-15T00-02:30:00,0[America/St_Johns]",
  "2019-12-15T00-02:30:00.000000000[America/St_Johns]",
  "2019-12-15T00-023000,0[America/St_Johns]",
  "2019-12-15T00-023000.000000000[America/St_Johns]",
];

for (const arg of validStrings) {
  const result = instance.since(arg);

  TemporalHelpers.assertDuration(
    result,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `"${arg}" is a valid UTC offset with time for PlainYearMonth`
  );
}

const invalidStrings = [
  "2022-09[u-ca=hebrew]",
  "2022-09Z",
  "2022-09+01:00",
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
    () => instance.since(arg),
    `"${arg}" UTC offset without time is not valid for PlainYearMonth`
  );
}
