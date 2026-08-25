// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tostring
description: Test printing Persian calendar dates as ISO strings
features: [Temporal]
---*/

const calendar = "persian";

const tests = [
  {
    testYear: 1395,
    isoDate: "2016-03-20",
  },
  {
    testYear: 1396,
    isoDate: "2017-03-21",
  },
  {
    testYear: 1397,
    isoDate: "2018-03-21",
  },
  {
    testYear: 1398,
    isoDate: "2019-03-21",
  },
  {
    testYear: 1399,
    isoDate: "2020-03-20",
  },
  {
    testYear: 1400,
    isoDate: "2021-03-21",
  }
];

for (let test of tests) {
  const date = Temporal.PlainDate.from({ year: test.testYear, month: 1, day: 1, calendar });
  const result = date.toString({ calendarName: "always" });
  assert.sameValue(result, `${test.isoDate}[u-ca=persian]`, `ISO reference date`);
}

