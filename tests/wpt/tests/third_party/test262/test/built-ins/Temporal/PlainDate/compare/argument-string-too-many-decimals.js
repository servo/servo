// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: No more than 9 decimal places are allowed
features: [Temporal]
---*/

var invalidStrings = [
  "1970-01-01T00:00:00.1234567891",
  "1970-01-01T00:00:00.1234567890",
  "1970-01-01T00+00:00:00.1234567891",
  "1970-01-01T00+00:00:00.1234567890",
];

invalidStrings.forEach(function (arg) {
  assert.throws(
    RangeError,
    function() { Temporal.PlainDate.compare(arg, new Temporal.PlainDate(1976, 11, 18)); },
    "no more than 9 decimal places are allowed (first arg)"
  );
  assert.throws(
    RangeError,
    function() { Temporal.PlainDate.compare(new Temporal.PlainDate(1976, 11, 18), arg); },
    "no more than 9 decimal places are allowed (second arg)"
  );
});
