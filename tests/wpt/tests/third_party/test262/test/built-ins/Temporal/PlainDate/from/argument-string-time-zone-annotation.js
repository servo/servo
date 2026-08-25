// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Various forms of time zone annotation; critical flag has no effect
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["2000-05-02[Asia/Kolkata]", "named, with no time"],
  ["2000-05-02[!Europe/Vienna]", "named, with ! and no time"],
  ["2000-05-02[+00:00]", "numeric, with no time"],
  ["2000-05-02[!-02:30]", "numeric, with ! and no time"],
  ["2000-05-02T15:23[America/Sao_Paulo]", "named, with no offset"],
  ["2000-05-02T15:23[!Asia/Tokyo]", "named, with ! and no offset"],
  ["2000-05-02T15:23[-02:30]", "numeric, with no offset"],
  ["2000-05-02T15:23[!+00:00]", "numeric, with ! and no offset"],
  ["2000-05-02T15:23+00:00[America/New_York]", "named, with offset"],
  ["2000-05-02T15:23+00:00[!UTC]", "named, with offset and !"],
  ["2000-05-02T15:23+00:00[+01:00]", "numeric, with offset"],
  ["2000-05-02T15:23+00:00[!-08:00]", "numeric, with offset and !"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.PlainDate.from(arg);

  TemporalHelpers.assertPlainDate(
    result,
    2000, 5, "M05", 2,
    `time zone annotation (${description})`
  );
});
