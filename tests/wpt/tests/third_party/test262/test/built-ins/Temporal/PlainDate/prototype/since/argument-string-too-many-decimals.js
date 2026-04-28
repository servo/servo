// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: No more than 9 decimal places are allowed
features: [Temporal]
---*/

var invalidStrings = [
  "1970-01-01T00:00:00.1234567891",
  "1970-01-01T00:00:00.1234567890",
  "1970-01-01T00+00:00:00.1234567891",
  "1970-01-01T00+00:00:00.1234567890",
];
var instance = new Temporal.PlainDate(2000, 5, 2);
invalidStrings.forEach(function (arg) {
  assert.throws(
    RangeError,
    function() { instance.since(arg); },
    "no more than 9 decimal places are allowed"
  );
});
