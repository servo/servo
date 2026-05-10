// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.constructor
description: An ISO string is not valid input for a constructor's time zone param
features: [Temporal]
---*/

assert.throws(
  RangeError,
  () => new Temporal.ZonedDateTime(0n, "1997-12-04T12:34[+01:00]", "iso8601"),
  "An ISO string is not a valid calendar ID for constructor parameter"
);
