// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: Various forms of time zone annotation; critical flag has no effect
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["1970-01-01T00:00Z[Asia/Kolkata]", "named, with Z"],
  ["1970-01-01T00:00Z[!Europe/Vienna]", "named, with Z and !"],
  ["1970-01-01T00:00Z[+00:00]", "numeric, with Z"],
  ["1970-01-01T00:00Z[!-02:30]", "numeric, with Z and !"],
  ["1970-01-01T00:00+00:00[UTC]", "named, with offset"],
  ["1970-01-01T00:00+00:00[!Africa/Abidjan]", "named, with offset and !"],
  ["1970-01-01T00:00+00:00[-08:00]", "numeric, with offset"],
  ["1970-01-01T00:00+00:00[!+01:00]", "numeric, with offset and !"],
];

const instance = new Temporal.Instant(0n);

tests.forEach(([arg, description]) => {
  const result = instance.since(arg);

  TemporalHelpers.assertDuration(
    result,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `time zone annotation (${description})`
  );
});
