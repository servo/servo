// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Various forms of time zone annotation; critical flag has no effect
features: [Temporal]
---*/

const tests = [
  ["1970-01-01T00:00[UTC]", "UTC", "named, with no offset"],
  ["1970-01-01T00:00[!UTC]", "UTC", "named, with ! and no offset"],
  ["1970-01-01T00:00[+00:00]", "+00:00", "numeric, with no offset"],
  ["1970-01-01T00:00[!+00:00]", "+00:00", "numeric, with ! and no offset"],
  ["1970-01-01T00:00Z[UTC]", "UTC", "named, with Z"],
  ["1970-01-01T00:00Z[!UTC]", "UTC", "named, with Z and !"],
  ["1970-01-01T00:00Z[+00:00]", "+00:00", "numeric, with Z"],
  ["1970-01-01T00:00Z[!+00:00]", "+00:00", "numeric, with Z and !"],
  ["1970-01-01T00:00+00:00[UTC]", "UTC", "named, with offset"],
  ["1970-01-01T00:00+00:00[!UTC]", "UTC", "named, with offset and !"],
  ["1970-01-01T00:00+00:00[+00:00]", "+00:00", "numeric, with offset"],
  ["1970-01-01T00:00+00:00[!+00:00]", "+00:00", "numeric, with offset and !"],
];

tests.forEach(([arg, expectedZone, description]) => {
  const instance = new Temporal.ZonedDateTime(0n, expectedZone);
  const result = instance.equals(arg);

  assert.sameValue(
    result,
    true,
    `time zone annotation (${description})`
  );
});
