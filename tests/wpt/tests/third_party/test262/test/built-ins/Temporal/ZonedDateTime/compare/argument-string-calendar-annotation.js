// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Various forms of calendar annotation; critical flag has no effect
features: [Temporal]
---*/

const tests = [
  ["1970-01-01T00:00[UTC][u-ca=iso8601]", "without !"],
  ["1970-01-01T00:00[UTC][!u-ca=iso8601]", "with !"],
  ["1970-01-01T00:00[UTC][u-ca=iso8601][u-ca=discord]", "second annotation ignored"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.ZonedDateTime.compare(arg, arg);

  assert.sameValue(
    result,
    0,
    `calendar annotation (${description})`
  );
});
