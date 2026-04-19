// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.compare
description: Various forms of calendar annotation; critical flag has no effect
features: [Temporal]
---*/

const tests = [
  ["1976-11-18T15:23[u-ca=iso8601]", "without time zone"],
  ["1976-11-18T15:23[UTC][u-ca=iso8601]", "with time zone"],
  ["1976-11-18T15:23[!u-ca=iso8601]", "with ! and no time zone"],
  ["1976-11-18T15:23[UTC][!u-ca=iso8601]", "with ! and time zone"],
  ["1976-11-18T15:23[u-ca=iso8601][u-ca=discord]", "second annotation ignored"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.PlainDateTime.compare(arg, arg);

  assert.sameValue(
    result,
    0,
    `calendar annotation (${description})`
  );
});
