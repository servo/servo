// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: No more than 9 decimal places are allowed
features: [Temporal]
---*/

var invalidStrings = [
  "1970-01-01T00:00:00.1234567891",
  "1970-01-01T00:00:00.1234567890",
  "1970-01-01T00+00:00:00.1234567891",
  "1970-01-01T00+00:00:00.1234567890",
  "00:00:00.1234567891",
  "00:00:00.1234567890",
  "00+00:00:00.1234567891",
  "00+00:00:00.1234567890",
];
const timeZone = "UTC";
var instance = new Temporal.ZonedDateTime(0n, timeZone);
invalidStrings.forEach(function (arg) {
  assert.throws(
    RangeError,
    function() { instance.withPlainTime(arg); },
    "no more than 9 decimal places are allowed"
  );
});
