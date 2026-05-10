// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Various forms of time zone annotation; critical flag has no effect
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["1970-01-01T00:00[UTC]", "named, with no offset"],
  ["1970-01-01T00:00[!UTC]", "named, with ! and no offset"],
  ["1970-01-01T00:00[+00:00]", "numeric, with no offset"],
  ["1970-01-01T00:00[!+00:00]", "numeric, with ! and no offset"],
  ["1970-01-01T00:00Z[UTC]", "named, with Z"],
  ["1970-01-01T00:00Z[!UTC]", "named, with Z and !"],
  ["1970-01-01T00:00Z[+00:00]", "numeric, with Z"],
  ["1970-01-01T00:00Z[!+00:00]", "numeric, with Z and !"],
  ["1970-01-01T00:00+00:00[UTC]", "named, with offset"],
  ["1970-01-01T00:00+00:00[!UTC]", "named, with offset and !"],
  ["1970-01-01T00:00+00:00[+00:00]", "numeric, with offset"],
  ["1970-01-01T00:00+00:00[!+00:00]", "numeric, with offset and !"],
];

const timeZone = "UTC";
const instance = new Temporal.ZonedDateTime(0n, timeZone);

tests.forEach(([arg, description]) => {
  const result = instance.since(arg);

  TemporalHelpers.assertDuration(
    result,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `time zone annotation (${description})`
  );
});
