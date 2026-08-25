// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.equals
description: Unknown annotations with critical flag are rejected
features: [Temporal]
---*/

const invalidStrings = [
  "1970-01-01[!foo=bar]",
  "1970-01-01T00:00[!foo=bar]",
  "1970-01-01T00:00[UTC][!foo=bar]",
  "1970-01-01T00:00[u-ca=iso8601][!foo=bar]",
  "1970-01-01T00:00[UTC][!foo=bar][u-ca=iso8601]",
  "1970-01-01T00:00[foo=bar][!_foo-bar0=Dont-Ignore-This-99999999999]",
];
const instance = new Temporal.PlainDate(2000, 5, 2);
invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => instance.equals(arg),
    `reject unknown annotation with critical flag: ${arg}`
  );
});
