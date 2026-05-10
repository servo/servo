// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: No more than 9 decimal places are allowed
features: [Temporal]
---*/

var invalidStrings = [
  "1970-01-01T00:00:00.1234567891Z",
  "1970-01-01T00:00:00.1234567890Z",
  "1970-01-01T00+00:00:00.1234567891",
  "1970-01-01T00+00:00:00.1234567890",
];

var epoch = new Temporal.Instant(0n);
invalidStrings.forEach(function (arg) {
  assert.throws(
    RangeError,
    function() { Temporal.Instant.compare(arg, epoch); },
    "no more than 9 decimal places are allowed (first arg)"
  );
  assert.throws(
    RangeError,
    function() { Temporal.Instant.compare(epoch, arg); },
    "no more than 9 decimal places are allowed (second arg)"
  );
});
