// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Various forms of time zone annotation; critical flag has no effect
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["2019-12-15T15:23[Asia/Kolkata]", "named, with no offset"],
  ["2019-12-15T15:23[!Europe/Vienna]", "named, with ! and no offset"],
  ["2019-12-15T15:23[+00:00]", "numeric, with no offset"],
  ["2019-12-15T15:23[!-02:30]", "numeric, with ! and no offset"],
  ["2019-12-15T15:23+00:00[UTC]", "named, with offset"],
  ["2019-12-15T15:23+00:00[!Africa/Abidjan]", "named, with offset and !"],
  ["2019-12-15T15:23+00:00[+01:00]", "numeric, with offset"],
  ["2019-12-15T15:23+00:00[!-08:00]", "numeric, with offset and !"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.PlainYearMonth.from(arg);

  TemporalHelpers.assertPlainYearMonth(
    result,
    2019, 12, "M12",
    `time zone annotation (${description})`
  );
});
