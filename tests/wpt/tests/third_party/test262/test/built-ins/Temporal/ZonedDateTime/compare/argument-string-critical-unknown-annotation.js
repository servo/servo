// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Unknown annotations with critical flag are rejected
features: [Temporal]
---*/

const invalidStrings = [
  "1970-01-01T00:00[UTC][!foo=bar]",
  "1970-01-01T00:00[UTC][!foo=bar][u-ca=iso8601]",
  "1970-01-01T00:00[UTC][u-ca=iso8601][!foo=bar]",
  "1970-01-01T00:00[foo=bar][!_foo-bar0=Dont-Ignore-This-99999999999]",
];

const datetime = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");

invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.compare(arg, datetime),
    `reject unknown annotation with critical flag: ${arg} (first argument)`
  );
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.compare(datetime, arg),
    `reject unknown annotation with critical flag: ${arg} (second argument)`
  );
});
