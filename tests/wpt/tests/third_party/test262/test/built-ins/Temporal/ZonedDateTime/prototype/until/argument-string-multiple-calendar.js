// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: >
  More than one calendar annotation is not syntactical if any have the criical
  flag
features: [Temporal]
---*/

const invalidStrings = [
  "1970-01-01T00:00[UTC][u-ca=iso8601][!u-ca=iso8601]",
  "1970-01-01T00:00[UTC][!u-ca=iso8601][u-ca=iso8601]",
  "1970-01-01T00:00[UTC][u-ca=iso8601][foo=bar][!u-ca=iso8601]",
];
const timeZone = "UTC";
const instance = new Temporal.ZonedDateTime(0n, timeZone);
invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => instance.until(arg),
    `reject more than one calendar annotation if any critical: ${arg}`
  );
});
